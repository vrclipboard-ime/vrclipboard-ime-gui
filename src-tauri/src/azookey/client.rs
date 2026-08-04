use std::{collections::HashSet, time::Duration};

use anyhow::{Context, Result};
use azookey_kkc::{
    Backend, Candidate, ConvertRequest, Converter, ConverterBuilder, InputStyle, LearningMode,
};
use platform_dirs::AppDirs;
use tauri::{AppHandle, Manager};
use tracing::{debug, info};

use crate::config::AzookeyBackend;

pub struct AzookeyConversionClient {
    converter: Converter,
    composing_text: String,
}

impl AzookeyConversionClient {
    pub fn new(app_handle: &AppHandle, backend: AzookeyBackend) -> Result<Self> {
        let resource_dir = app_handle
            .path()
            .resource_dir()
            .context("failed to resolve Tauri resource directory")?;
        let native_dir = resource_dir.join("azookey-native");
        let model_path = resource_dir.join("ggml-model-Q5_K_M.gguf");

        let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false)
            .context("failed to resolve application data directories")?;
        let data_dir = app_dirs.config_dir.join("AzooKey");
        let backend = match backend {
            AzookeyBackend::Cpu => Backend::Cpu,
            AzookeyBackend::Vulkan => Backend::Vulkan,
        };

        info!(
            ?backend,
            native_dir = %native_dir.display(),
            model_path = %model_path.display(),
            "initializing AzooKey converter"
        );
        let converter = ConverterBuilder::new(backend, model_path)
            .native_dir(native_dir)
            .memory_directory(data_dir.join("memory"))
            .shared_container(data_dir.join("shared"))
            .learning_mode(LearningMode::Disabled)
            .preload_dictionary(true)
            .n_best(10)
            .inference_limit(10)
            .timeout(Duration::from_secs(300))
            .build()
            .context("failed to initialize azookey-kkc")?;

        Ok(Self {
            converter,
            composing_text: String::new(),
        })
    }

    pub fn backend(&self) -> Backend {
        self.converter.backend()
    }

    pub fn reset_composing_text(&mut self) {
        self.composing_text.clear();
    }

    pub fn insert_at_cursor_position(&mut self, text: &str) {
        self.composing_text.push_str(&pre_process_text(text));
    }

    pub fn request_candidates(&self, context: &str) -> Result<Vec<Candidate>> {
        debug!(context, text = %self.composing_text, "requesting AzooKey candidates");
        let mut request =
            ConvertRequest::new(&self.composing_text).input_style(InputStyle::Roman2Kana);
        if !context.is_empty() {
            request.options.left_side_context = Some(context.to_owned());
        }
        let result = self
            .converter
            .convert(request)
            .context("AzooKey conversion failed")?;
        Ok(post_process_candidates(result.main_results))
    }
}

fn pre_process_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 1);
    for character in text.chars() {
        result.push_str(match character {
            '-' => "ー",
            '=' => "＝",
            '[' => "「",
            ']' => "」",
            ';' => "；",
            '@' => "＠",
            ',' => "、",
            '.' => "。",
            '/' => "・",
            '!' => "！",
            '#' => "＃",
            '$' => "＄",
            '%' => "％",
            '^' => "＾",
            '&' => "＆",
            '*' => "＊",
            '(' => "（",
            ')' => "）",
            '_' => "＿",
            '+' => "＋",
            '{' => "｛",
            '}' => "｝",
            '|' => "｜",
            ':' => "：",
            '"' => "”",
            '<' => "＜",
            '>' => "＞",
            '?' => "？",
            '\\' => "￥",
            _ => {
                result.push(character);
                continue;
            }
        });
    }

    if result.ends_with('n') {
        let mut characters = result.chars().rev();
        characters.next();
        if characters.next().is_some_and(|previous| previous != 'n') {
            result.push('n');
        }
    }
    result.push('§');
    result
}

fn post_process_candidates(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .take(8)
        .filter_map(|mut candidate| {
            if candidate.text.ends_with('§') {
                candidate.text.pop();
            }
            seen.insert(candidate.text.clone()).then_some(candidate)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::pre_process_text;

    #[test]
    fn preprocessing_preserves_roman_input_and_normalizes_symbols() {
        assert_eq!(pre_process_text("konnichiha?"), "konnichiha？§");
        assert_eq!(pre_process_text("kan"), "kann§");
        assert_eq!(pre_process_text("kann"), "kann§");
    }
}
