use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use azookey_binding::{Candidate, ComposingText, KanaKanjiConverter};
use ipc_channel::ipc::IpcOneShotServer;
use itertools::Itertools;
use platform_dirs::AppDirs;

use crate::SELF_EXE_PATH;

use super::IpcMessage;

static SIGNMAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("-", "ー"),
        ("=", "＝"),
        ("[", "「"),
        ("]", "」"),
        (";", "；"),
        ("@", "＠"),
        (",", "、"),
        (".", "。"),
        ("/", "・"),
        ("!", "！"),
        ("#", "＃"),
        ("$", "＄"),
        ("%", "％"),
        ("^", "＾"),
        ("&", "＆"),
        ("*", "＊"),
        ("(", "（"),
        (")", "）"),
        ("_", "＿"),
        ("+", "＋"),
        ("{", "｛"),
        ("}", "｝"),
        ("|", "｜"),
        (":", "："),
        ("\"", "”"),
        ("<", "＜"),
        (">", "＞"),
        ("?", "？"),
        ("\\", "￥"),
    ])
});

pub struct AzookeyConversionServer {
    pub azookey_converter: KanaKanjiConverter,
    pub composing_text: ComposingText,
    pub server: IpcOneShotServer<IpcMessage>,
    pub server_name: String,
}

impl AzookeyConversionServer {
    pub fn new() -> Self {
        let (server, server_name): (IpcOneShotServer<IpcMessage>, String) =
            IpcOneShotServer::new().unwrap();

        let instance = Self {
            azookey_converter: KanaKanjiConverter::new(),
            composing_text: ComposingText::new(),
            server,
            server_name,
        };
        instance
    }

    fn pre_process_text(text: &str) -> String {
        let mut result = String::new();

        // replace all characters in the text with their corresponding replacements
        for c in text.chars() {
            if let Some(&replacement) = SIGNMAP.get(c.to_string().as_str()) {
                result.push_str(replacement);
            } else {
                result.push(c);
            }
        }

        // push 'n' if the last and second last characters are 'n'
        if result.ends_with('n') {
            let mut chars = result.chars().collect::<Vec<_>>();
            if chars.len() > 1 && chars[chars.len() - 2] != 'n' {
                chars.push('n');
            }
            result = chars.into_iter().collect::<String>();
        }

        // push '§' at the end of the string to avoid unnecessary prediction
        result.push_str("§");

        result
    }

    fn post_process_text(text: &str) -> String {
        let mut result = text.to_string();

        if result.ends_with('§') {
            result.pop();
        }

        result
    }

    fn post_process_candidates(candidates: Vec<Candidate>) -> Vec<Candidate> {
        candidates
            .iter()
            .take(8)
            .map(|c| {
                let mut candidate = c.clone();
                candidate.text = Self::post_process_text(&candidate.text);
                candidate
            })
            .unique_by(|c| c.text.clone())
            .collect()
    }

    pub fn server_loop(mut self) {
        let (a, _) = self.server.accept().unwrap();
        let mut sender = None;

        let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false).unwrap();
        let path = app_dirs
            .config_dir
            .join("AzooKeyDictionary\\AzooKeyDictionary\\Dictionary");
        let extract_path = path.to_str().unwrap();

        println!("Extract path: {}", extract_path);

        let self_exe_path = PathBuf::from(SELF_EXE_PATH.read().unwrap().as_str());
        let weight_path = self_exe_path
            .parent()
            .unwrap()
            .join("ggml-model-Q5_K_M.gguf")
            .to_string_lossy()
            .to_string();

        loop {
            match a.recv() {
                Ok(IpcMessage::Sender(s)) => {
                    sender = Some(s);
                }
                Ok(IpcMessage::ResetComposingText) => {
                    self.composing_text = ComposingText::new();
                }
                Ok(IpcMessage::InsertAtCursorPosition(text)) => {
                    let text = Self::pre_process_text(&text);
                    self.composing_text.insert_at_cursor_position(&text);
                }
                Ok(IpcMessage::RequestCandidates(context)) => {
                    let candidates = self.azookey_converter.request_candidates(
                        &self.composing_text,
                        &context,
                        &extract_path,
                        &weight_path,
                    );
                    if let Some(s) = sender.as_ref() {
                        let candidates = Self::post_process_candidates(candidates);
                        s.send(IpcMessage::Candidates(candidates)).unwrap();
                    }
                }
                Ok(IpcMessage::End) => {
                    break;
                }
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
    }
}
