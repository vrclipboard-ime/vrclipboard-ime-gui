use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use azookey_binding::{Candidate, ComposingText, KanaKanjiConverter};
use platform_dirs::AppDirs;
use tracing::info;

use crate::SELF_EXE_PATH;

use super::processing;

struct ConverterState {
    azookey_converter: KanaKanjiConverter,
    composing_text: ComposingText,
    dictionary_path: String,
    weight_path: String,
}

impl ConverterState {
    fn new() -> Self {
        info!("Initializing DirectAzookeyConverter");

        let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false).unwrap();
        let dictionary_path = app_dirs
            .config_dir
            .join("AzooKeyDictionary")
            .join("AzooKeyDictionary")
            .join("Dictionary")
            .to_string_lossy()
            .to_string();

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
            dictionary_path,
            weight_path,
        }
    }
}

pub struct DirectAzookeyConverter {
    state: Mutex<ConverterState>,
}

impl DirectAzookeyConverter {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ConverterState::new()),
        }
    }

    pub fn reset_composing_text(&self) {
        let mut state = self.state.lock().unwrap();
        state.composing_text = ComposingText::new();
    }

    pub fn insert_at_cursor_position(&self, text: &str) {
        let processed_text = processing::pre_process_text(text);
        let state = self.state.lock().unwrap();
        state
            .composing_text
            .insert_at_cursor_position(&processed_text);
    }

    pub fn request_candidates(&self, context: &str) -> Vec<Candidate> {
        println!(
            "dict: {}, weight: {}",
            self.state.lock().unwrap().dictionary_path,
            self.state.lock().unwrap().weight_path,
        );
        let state = self.state.lock().unwrap();
        let candidates = state.azookey_converter.request_candidates(
            &state.composing_text,
            context,
            &state.dictionary_path,
            &state.weight_path,
        );
        processing::post_process_candidates(candidates)
    }
}

// Safety: The mutex ensures exclusive access to C FFI objects
unsafe impl Send for DirectAzookeyConverter {}
unsafe impl Sync for DirectAzookeyConverter {}

static GLOBAL_CONVERTER: OnceLock<Arc<DirectAzookeyConverter>> = OnceLock::new();

pub fn get_global_converter() -> Arc<DirectAzookeyConverter> {
    GLOBAL_CONVERTER
        .get_or_init(|| Arc::new(DirectAzookeyConverter::new()))
        .clone()
}
