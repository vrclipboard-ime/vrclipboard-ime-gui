use anyhow::{anyhow, Result};
use tracing::{debug, info, trace};

use super::client::AzookeyConversionClient;

/// Maximum number of history entries to retain
const MAX_HISTORY_SIZE: usize = 3;

/// Maximum number of conversion candidates
const MAX_CANDIDATES: usize = 10;

/// AzookeyConversion - Provides romanized text to kanji conversion and candidate switching
///
/// This struct implements the logic for character conversion and candidate switching
/// in a Japanese input method system.
pub struct AzookeyConversion {
    /// Conversion history (max 3 entries)
    conversion_history: Vec<String>,

    /// Pre-conversion text history (max 3 entries)
    input_history: Vec<String>,

    /// Whether currently in reconversion mode
    is_reconversion_mode: bool,

    /// Current text being converted
    current_text: String,

    /// List of reconversion candidates
    reconversion_candidates: Option<Vec<String>>,

    /// Index of currently selected reconversion candidate
    candidate_index: Option<usize>,

    /// Common prefix for reconversion
    common_prefix: Option<String>,

    /// Client that performs conversion operations
    client: AzookeyConversionClient,
}

impl AzookeyConversion {
    /// Creates a new AzookeyConversion instance
    ///
    /// # Arguments
    /// * `client` - Client instance that performs conversion operations
    ///
    /// # Returns
    /// * Initialized AzookeyConversion instance
    pub fn new(client: AzookeyConversionClient) -> Self {
        info!("Creating new AzookeyConversion instance");

        Self {
            conversion_history: Vec::new(),
            input_history: Vec::new(),
            is_reconversion_mode: false,
            current_text: String::new(),
            reconversion_candidates: None,
            candidate_index: None,
            common_prefix: None,
            client,
        }
    }

    /// Converts text - Main entry point for conversion processing
    ///
    /// # Arguments
    /// * `text` - Text to be converted
    ///
    /// # Returns
    /// * `Result<String>` - Conversion result or error
    pub fn convert(&mut self, text: &str) -> Result<String> {
        debug!("Starting conversion: {}", text);
        trace!(
            "Current state: conversion_history={:?}, input_history={:?}, is_reconversion_mode={}",
            self.conversion_history,
            self.input_history,
            self.is_reconversion_mode
        );

        self.current_text = text.to_string();

        // Check if same as previous conversion result
        let same_as_last_conversion = self.is_same_as_last_conversion(text);
        trace!("Same as last conversion: {}", same_as_last_conversion);

        // Reset if input changed while in reconversion mode
        if !same_as_last_conversion && self.is_reconversion_mode {
            debug!("Resetting conversion state due to new input");
            self.reset_reconversion_state();
        }

        // Branch conversion processing
        if self.is_reconversion_mode || same_as_last_conversion {
            // Same text re-entered or in reconversion mode
            info!("Executing AzooKey conversion");
            self.convert_with_candidates(text)
        } else {
            // Normal conversion processing
            info!("Executing regular romaji->kanji conversion");
            self.convert_roman_to_kanji(text)
        }
    }

    /// Determines if input is the same as the last conversion result
    ///
    /// # Arguments
    /// * `text` - Text to check
    ///
    /// # Returns
    /// * `bool` - True if same as last conversion
    fn is_same_as_last_conversion(&self, text: &str) -> bool {
        if let Some(last_conversion) = self.conversion_history.last() {
            text == last_conversion
        } else {
            false
        }
    }

    /// Resets reconversion-related state
    fn reset_reconversion_state(&mut self) {
        debug!("Resetting reconversion state");

        trace!(
            "Before reset - is_reconversion_mode: {}, common_prefix: {:?}, candidate_index: {:?}",
            self.is_reconversion_mode,
            self.common_prefix,
            self.candidate_index
        );

        self.is_reconversion_mode = false;
        self.common_prefix = None;
        self.candidate_index = None;
        self.reconversion_candidates = None;

        trace!(
            "After reset - is_reconversion_mode: {}, common_prefix: {:?}, candidate_index: {:?}",
            self.is_reconversion_mode,
            self.common_prefix,
            self.candidate_index
        );
    }

    /// Converts romaji to kanji (initial conversion)
    ///
    /// # Arguments
    /// * `text` - Text to convert
    ///
    /// # Returns
    /// * `Result<String>` - Conversion result or error
    fn convert_roman_to_kanji(&mut self, text: &str) -> Result<String> {
        debug!("Converting romaji to kanji: {}", text);

        // Get previous conversion result
        let previous_conversion = self.get_previous_conversion();
        trace!("Previous conversion: {}", previous_conversion);

        // Detect difference position
        let first_diff_position = self.find_first_difference(&previous_conversion, text);
        trace!("First difference position: {:?}", first_diff_position);

        // Extract difference text
        let diff_text = text.chars().skip(first_diff_position).collect::<String>();
        debug!("Difference to convert: {}", diff_text);

        // Extract common prefix
        let prefix = text.chars().take(first_diff_position).collect::<String>();
        trace!("Conversion prefix: {}", prefix);

        // Use client for conversion
        self.client.reset_composing_text();
        self.client.insert_at_cursor_position(&diff_text);

        // Get and select conversion candidate
        let converted = match self.client.request_candidates("").first() {
            Some(candidate) => candidate.text.clone(),
            None => return Err(anyhow!("No conversion candidates available")),
        };
        trace!("Conversion result: {}", converted);

        // Update history
        let result = prefix + &converted;
        self.update_history(result.clone(), text.to_string());

        info!("Romaji->kanji conversion result: {}", result);
        Ok(result)
    }

    /// Switches between conversion candidates (reconversion)
    ///
    /// # Arguments
    /// * `text` - Text for reconversion
    ///
    /// # Returns
    /// * `Result<String>` - Conversion result or error
    fn convert_with_candidates(&mut self, text: &str) -> Result<String> {
        debug!("Converting with AzooKey: {}", text);
        self.is_reconversion_mode = true;

        // Prepare for reconversion if needed
        self.prepare_reconversion_if_needed();

        // Select and switch candidates
        let result = self.select_next_candidate()?;

        // Update history
        self.update_history(result.clone(), text.to_string());

        info!("AzooKey conversion result: {}", result);
        Ok(result)
    }

    /// Prepares difference processing for reconversion
    fn prepare_reconversion_if_needed(&mut self) {
        // Only execute on first reconversion
        if self.common_prefix.is_none() {
            debug!("Preparing for reconversion");

            // Calculate differences from past history
            let previous_output = self.get_previous_output(2);
            let previous_input = self.get_previous_input(1);
            trace!(
                "Previous output: {}, previous input: {}",
                previous_output,
                previous_input
            );

            // Detect difference position
            let first_diff_position = self.find_first_difference(&previous_input, &previous_output);
            trace!("First difference position: {:?}", first_diff_position);

            // Extract difference text
            let diff_text = previous_input
                .chars()
                .skip(first_diff_position)
                .collect::<String>();
            debug!("Reconversion difference: {}", diff_text);

            // Set common prefix
            let prefix = previous_input
                .chars()
                .take(first_diff_position)
                .collect::<String>();
            self.common_prefix = Some(prefix.clone());
            trace!("Set reconversion prefix: {}", prefix);

            // Generate candidates
            self.generate_candidates(&diff_text, &prefix);
        }
    }

    /// Generates conversion candidates
    ///
    /// # Arguments
    /// * `diff_text` - Difference text to convert
    /// * `prefix` - Common prefix
    fn generate_candidates(&mut self, diff_text: &str, prefix: &str) {
        debug!("Generating candidates");

        self.client.reset_composing_text();
        self.client.insert_at_cursor_position(diff_text);

        // Get candidates from client
        let mut candidates = self
            .client
            .request_candidates(prefix)
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<String>>();
        trace!("Retrieved candidates: {:?}", candidates);

        // Include raw text in candidates
        candidates.insert(0, diff_text.to_string());

        // Limit number of candidates (for safety)
        if candidates.len() > MAX_CANDIDATES {
            candidates.truncate(MAX_CANDIDATES);
        }

        trace!("Final candidate list: {:?}", candidates);
        self.reconversion_candidates = Some(candidates);
        self.candidate_index = Some(0);
    }

    /// Selects the next candidate
    ///
    /// # Returns
    /// * `Result<String>` - Selected candidate or error
    fn select_next_candidate(&mut self) -> Result<String> {
        let candidates = match &self.reconversion_candidates {
            Some(cands) => cands,
            None => return Err(anyhow!("Candidate list does not exist")),
        };

        let index = match self.candidate_index {
            Some(i) => {
                // Update index
                let new_index = if i + 1 < candidates.len() { i + 1 } else { 0 };
                self.candidate_index = Some(new_index);
                new_index
            }
            None => return Err(anyhow!("Candidate index not initialized")),
        };

        debug!("Updated candidate index: {}", index);

        // Get selected candidate
        let prefix = self.common_prefix.clone().unwrap_or_default();
        let selected_candidate = &candidates[index];
        let result = prefix + selected_candidate;

        Ok(result)
    }

    /// Updates history
    ///
    /// # Arguments
    /// * `conversion` - Conversion result
    /// * `input` - Input text
    fn update_history(&mut self, conversion: String, input: String) {
        self.conversion_history.push(conversion);
        self.input_history.push(input);

        trace!(
            "Before trim: conversion_history={}, input_history={}",
            self.conversion_history.len(),
            self.input_history.len()
        );

        // Limit history size
        self.trim_history();

        trace!(
            "After trim: conversion_history={}, input_history={}",
            self.conversion_history.len(),
            self.input_history.len()
        );
    }

    /// Limits history size
    fn trim_history(&mut self) {
        while self.conversion_history.len() > MAX_HISTORY_SIZE {
            self.conversion_history.remove(0);
        }

        while self.input_history.len() > MAX_HISTORY_SIZE {
            self.input_history.remove(0);
        }
    }

    /// Gets previous conversion result
    ///
    /// # Returns
    /// * `String` - Previous conversion result or empty string
    fn get_previous_conversion(&self) -> String {
        self.conversion_history
            .last()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Gets conversion history from specified index
    ///
    /// # Arguments
    /// * `offset` - Offset from the end
    ///
    /// # Returns
    /// * `String` - Retrieved history or empty string
    fn get_previous_output(&self, offset: usize) -> String {
        if offset <= self.conversion_history.len() {
            self.conversion_history[self.conversion_history.len() - offset].clone()
        } else {
            String::new()
        }
    }

    /// Gets input history from specified index
    ///
    /// # Arguments
    /// * `offset` - Offset from the end
    ///
    /// # Returns
    /// * `String` - Retrieved history or empty string
    fn get_previous_input(&self, offset: usize) -> String {
        if offset <= self.input_history.len() {
            self.input_history[self.input_history.len() - offset].clone()
        } else {
            String::new()
        }
    }

    /// Finds first difference position between two strings
    ///
    /// # Arguments
    /// * `s1` - First string to compare
    /// * `s2` - Second string to compare
    ///
    /// # Returns
    /// * `usize` - First difference position
    fn find_first_difference(&self, s1: &str, s2: &str) -> usize {
        let result = s1
            .chars()
            .zip(s2.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| {
                // If one is a prefix of the other
                let min_len = s1.chars().count().min(s2.chars().count());
                if s1.len() != s2.len() {
                    min_len
                } else {
                    0
                }
            });

        trace!(
            "String comparison: \"{}\" and \"{}\" differ at position: {}",
            s1,
            s2,
            result
        );
        result
    }
}
