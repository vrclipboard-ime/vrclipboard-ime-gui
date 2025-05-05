use anyhow::Result;
use tracing::{debug, error, info, trace};

use super::client::AzookeyConversionClient;

pub struct AzookeyConversion {
    pub conversion_history: Vec<String>,
    pub clipboard_history: Vec<String>,
    pub now_reconvertion: bool,
    pub target_text: String,
    pub reconversion_candidates: Option<Vec<String>>,
    pub reconversion_index: Option<i32>,
    pub reconversion_prefix: Option<String>,
    pub client: AzookeyConversionClient,
}

impl AzookeyConversion {
    pub fn new(client: AzookeyConversionClient) -> Self {
        info!("Creating new AzookeyConversion instance");

        let instance = Self {
            conversion_history: Vec::new(),
            clipboard_history: Vec::new(),
            now_reconvertion: false,
            target_text: String::new(),
            reconversion_candidates: None,
            reconversion_index: None,
            reconversion_prefix: None,
            client,
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

        let prefix = text
            .chars()
            .take(first_diff_position.unwrap_or(0))
            .collect::<String>();
        trace!("Prefix for conversion: {}", prefix);
        self.client.reset_composing_text();
        self.client.insert_at_cursor_position(&diff);
        trace!("target: {}, prefix: {}", diff, prefix);
        let converted = self
            .client
            .request_candidates("")
            .first()
            .unwrap()
            .text
            .clone();
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
            self.client.reset_composing_text();
            self.client.insert_at_cursor_position(&diff);
            let prefix = self.reconversion_prefix.clone().unwrap_or_default();
            let mut candidates = self
                .client
                .request_candidates(&prefix)
                .iter()
                .map(|c| c.text.clone())
                .collect::<Vec<String>>();
            trace!("Initial candidates: {:?}", candidates);
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
