#[cfg(target_os = "windows")]
use crate::{
    converter::{
        converter::Converter, hiragana::HiraganaConverter, roman_to_kanji::RomanToKanjiConverter,
    },
    tsf::{search_candidate_provider::SearchCandidateProvider, set_thread_local_input_settings},
};
use anyhow::Result;
use tracing::{debug, error, info, trace};

#[cfg(target_os = "windows")]
pub struct TsfConversion {
    pub conversion_history: Vec<String>,
    pub clipboard_history: Vec<String>,
    pub now_reconvertion: bool,
    pub target_text: String,
    pub search_candidate_provider: SearchCandidateProvider,
    pub reconversion_candidates: Option<Vec<String>>,
    pub reconversion_index: Option<i32>,
    pub reconversion_prefix: Option<String>,
}

#[cfg(target_os = "windows")]
impl TsfConversion {
    pub fn new() -> Self {
        info!("Creating new TsfConversion instance");
        set_thread_local_input_settings(true).unwrap();

        let instance = Self {
            conversion_history: Vec::new(),
            clipboard_history: Vec::new(),
            now_reconvertion: false,
            target_text: String::new(),
            search_candidate_provider: SearchCandidateProvider::create().unwrap(),
            reconversion_candidates: None,
            reconversion_index: None,
            reconversion_prefix: None,
        };
        instance
    }

    fn reset_conversion_state(&mut self) {
        debug!("Resetting conversion state");
        trace!("Before reset - now_reconvertion: {}, reconversion_prefix: {:?}, reconversion_index: {:?}, reconversion_candidates: {:?}",
               self.now_reconvertion, self.reconversion_prefix, self.reconversion_index, self.reconversion_candidates);
        self.now_reconvertion = false;
        self.reconversion_prefix = None;
        self.reconversion_index = None;
        self.reconversion_candidates = None;
        trace!("After reset - now_reconvertion: {}, reconversion_prefix: {:?}, reconversion_index: {:?}, reconversion_candidates: {:?}",
               self.now_reconvertion, self.reconversion_prefix, self.reconversion_index, self.reconversion_candidates);
    }

    fn convert_roman_to_kanji(&mut self, text: &str) -> Result<String> {
        debug!("Converting roman to kanji: {}", text);
        let o_minus_1 = self
            .conversion_history
            .get(if self.conversion_history.len() > 0 {
                self.conversion_history.len() - 1
            } else {
                0
            })
            .unwrap_or(&("".to_string()))
            .clone();
        trace!("Previous conversion (o_minus_1): {}", o_minus_1);
        let mut first_diff_position = o_minus_1
            .chars()
            .zip(text.chars())
            .position(|(a, b)| a != b);

        if o_minus_1 != text && first_diff_position.is_none() {
            first_diff_position = Some(o_minus_1.chars().count());
        }
        trace!("First difference position: {:?}", first_diff_position);
        let diff = text
            .chars()
            .skip(first_diff_position.unwrap_or(0))
            .collect::<String>();
        debug!("Difference to convert: {}", diff);

        let roman_to_kanji_converter = RomanToKanjiConverter;
        let converted = roman_to_kanji_converter.convert(&diff)?;
        trace!("Converted difference: {}", converted);
        self.conversion_history.push(
            o_minus_1
                .chars()
                .zip(text.chars())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect::<String>()
                + &converted,
        );
        self.clipboard_history.push(text.to_string());
        info!(
            "Roman to kanji conversion result: {}",
            self.conversion_history.last().unwrap()
        );
        trace!("Updated conversion history: {:?}", self.conversion_history);
        trace!("Updated clipboard history: {:?}", self.clipboard_history);
        Ok(self.conversion_history.last().unwrap().clone())
    }

    fn convert_tsf(&mut self, text: &str) -> Result<String> {
        debug!("Converting using TSF: {}", text);
        self.now_reconvertion = true;
        let mut diff_hiragana = String::new();
        let mut diff = String::new();
        if self.reconversion_prefix.is_none() {
            let o_minus_2 = self
                .conversion_history
                .get(if self.conversion_history.len() > 1 {
                    self.conversion_history.len() - 2
                } else {
                    0
                })
                .unwrap_or(&("".to_string()))
                .clone();
            let i_minus_1 = self
                .clipboard_history
                .get(if self.clipboard_history.len() > 0 {
                    self.clipboard_history.len() - 1
                } else {
                    0
                })
                .unwrap_or(&("".to_string()))
                .clone();
            trace!("o_minus_2: {}, i_minus_1: {}", o_minus_2, i_minus_1);
            let mut first_diff_position = i_minus_1
                .chars()
                .zip(o_minus_2.chars())
                .position(|(a, b)| a != b);
            trace!("First difference position: {:?}", first_diff_position);
            if o_minus_2 != i_minus_1 && first_diff_position.is_none() {
                first_diff_position = Some(o_minus_2.chars().count());
            }
            diff = i_minus_1
                .chars()
                .skip(first_diff_position.unwrap_or(0))
                .collect::<String>();
            debug!("Difference to convert: {}", diff);
            diff_hiragana = HiraganaConverter.convert(&diff)?;
            trace!("Hiragana conversion: {}", diff_hiragana);
            let prefix = i_minus_1
                .chars()
                .zip(o_minus_2.chars())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect::<String>();
            self.reconversion_prefix = Some(prefix.clone());
            trace!("Set reconversion prefix: {:?}", self.reconversion_prefix);
        }

        let candidates = self.reconversion_candidates.get_or_insert_with(|| {
            debug!("Generating new candidates");
            let mut candidates = self
                .search_candidate_provider
                .get_candidates(&diff_hiragana, 10)
                .unwrap_or_default();
            trace!("Initial candidates: {:?}", candidates);
            if candidates.is_empty() {
                candidates.push(diff_hiragana.clone());
                let roman_to_kanji_converter = RomanToKanjiConverter;
                let roman_to_kanji = roman_to_kanji_converter.convert(&diff_hiragana).unwrap();
                candidates.push(roman_to_kanji);
            }
            candidates.insert(0, diff.to_string());
            trace!("Final candidates: {:?}", candidates);
            candidates
        });

        let index = self.reconversion_index.get_or_insert(-1);
        trace!("Current reconversion index: {}", index);

        if *index + 1 < candidates.len() as i32 {
            *index += 1;
        } else {
            *index = 0;
        }
        debug!("Updated reconversion index: {}", index);

        self.conversion_history.push(
            self.reconversion_prefix.clone().unwrap()
                + &self.reconversion_candidates.as_ref().unwrap()
                    [self.reconversion_index.unwrap() as usize]
                    .clone(),
        );
        self.clipboard_history.push(text.to_string());
        trace!("Updated conversion history: {:?}", self.conversion_history);
        trace!("Updated clipboard history: {:?}", self.clipboard_history);

        while self.conversion_history.len() > 3 {
            self.conversion_history.remove(0);
        }
        while self.clipboard_history.len() > 3 {
            self.clipboard_history.remove(0);
        }
        trace!("Trimmed conversion history: {:?}", self.conversion_history);
        trace!("Trimmed clipboard history: {:?}", self.clipboard_history);

        info!(
            "TSF conversion result: {}",
            self.conversion_history.last().unwrap()
        );
        Ok(self.conversion_history.last().unwrap().clone())
    }

    pub fn convert(&mut self, text: &str) -> Result<String> {
        debug!("Starting conversion for: {}", text);
        trace!("Current conversion history: {:?}", self.conversion_history);
        trace!("Current clipboard history: {:?}", self.clipboard_history);
        let same_as_last_conversion = text.to_string()
            == self
                .conversion_history
                .last()
                .unwrap_or(&("".to_string()))
                .clone();
        trace!("Same as last conversion: {}", same_as_last_conversion);

        self.target_text = text.to_string();
        trace!("Set target text: {}", self.target_text);

        if !same_as_last_conversion && self.now_reconvertion {
            debug!("Resetting conversion state due to new input");
            self.reset_conversion_state();
        }

        if !self.now_reconvertion && !same_as_last_conversion {
            info!("Converting using roman_to_kanji");
            return self.convert_roman_to_kanji(text);
        }

        if same_as_last_conversion || self.now_reconvertion {
            info!("Converting using TSF");
            return self.convert_tsf(text);
        }

        error!("Failed to convert: {}", text);
        Err(anyhow::anyhow!("Failed to convert"))
    }
}

#[cfg(not(target_os = "windows"))]
pub struct TsfConversion;

#[cfg(not(target_os = "windows"))]
impl TsfConversion {
    pub fn new() -> Self {
        Self
    }
    pub fn convert(&mut self, _text: &str) -> Result<String> {
        Err(anyhow::anyhow!(
            "TsfConversion is only supported on Windows."
        ))
    }
}
