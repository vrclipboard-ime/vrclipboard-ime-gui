use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use azookey_binding::{Candidate, ComposingText, KanaKanjiConverter};
use ipc_channel::ipc::{IpcReceiver, IpcSender};

use itertools::Itertools;
use platform_dirs::AppDirs;
use tracing::info;

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

pub struct AzookeyConversionClient {
    pub azookey_converter: KanaKanjiConverter,
    pub composing_text: ComposingText,
    pub extract_path: String,
    pub weight_path: String,
}

impl AzookeyConversionClient {
    pub fn new() -> Self {
        println!("Creating new AzookeyConversionClient instance");

        let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false).unwrap();
        let path = app_dirs
            .config_dir
            .join("AzooKeyDictionary/AzooKeyDictionary/Dictionary");
        let extract_path = path.to_str().unwrap();

        println!("Extract path: {}", extract_path);

        let self_exe_path = PathBuf::from(SELF_EXE_PATH.read().unwrap().as_str());
        let weight_path = self_exe_path
            .parent()
            .unwrap()
            .join("ggml-model-Q5_K_M.gguf")
            .to_string_lossy()
            .to_string();

        Self {
            azookey_converter: KanaKanjiConverter::new(),
            composing_text: ComposingText::new(),
            extract_path: extract_path.to_string(),
            weight_path,
        }
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

    pub fn reset_composing_text(&mut self) {
        info!("Resetting composing text");

        self.composing_text = ComposingText::new();
    }

    pub fn insert_at_cursor_position(&mut self, text: &str) {
        info!("Inserting at cursor position: {}", text);

        let text = Self::pre_process_text(text);
        self.composing_text.insert_at_cursor_position(&text);
    }

    pub fn request_candidates(&mut self, context: &str) -> Vec<Candidate> {
        info!("Requesting candidates for context: {}", context);
        info!("Dictionary extract path: {}", self.extract_path);
        info!("Model weight path: {}", self.weight_path);
        let candidates = self.azookey_converter.request_candidates(
            &self.composing_text,
            context,
            &self.extract_path,
            &self.weight_path,
        );
        println!("{:?}", candidates);
        Self::post_process_candidates(candidates)
    }
}
