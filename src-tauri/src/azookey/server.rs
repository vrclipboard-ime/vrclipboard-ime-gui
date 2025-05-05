use std::{collections::HashMap, sync::LazyLock};

use azookey_binding::{Candidate, ComposingText, KanaKanjiConverter};
use ipc_channel::ipc::IpcOneShotServer;
use platform_dirs::AppDirs;

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

    pub fn server_loop(mut self) {
        let (a, _) = self.server.accept().unwrap();
        let mut sender = None;

        loop {
            match a.recv() {
                Ok(IpcMessage::Sender(s)) => {
                    sender = Some(s);
                }
                Ok(IpcMessage::ResetComposingText) => {
                    self.composing_text = ComposingText::new();
                }
                Ok(IpcMessage::InsertAtCursorPosition(text)) => {
                    let text: String = text
                        .chars()
                        .map(|c| {
                            if let Some(&replacement) = SIGNMAP.get(c.to_string().as_str()) {
                                return replacement.to_string();
                            } else {
                                c.to_string()
                            }
                        })
                        .collect();
                    self.composing_text.insert_at_cursor_position(&text);
                }
                Ok(IpcMessage::RequestCandidates(context, weight_path)) => {
                    let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false).unwrap();
                    let path = app_dirs
                        .config_dir
                        .join("AzooKeyDictionary\\AzooKeyDictionary\\Dictionary");
                    let extract_path = path.to_str().unwrap();
                    let candidates = self.azookey_converter.request_candidates(
                        &self.composing_text,
                        &context,
                        &extract_path,
                        &weight_path,
                    );
                    if let Some(s) = sender.as_ref() {
                        let candidates = candidates
                            .iter()
                            .take(8)
                            .map(|c| c.clone())
                            .collect::<Vec<Candidate>>();
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
