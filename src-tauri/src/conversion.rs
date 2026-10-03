use crate::{
    STATE,
    config::Config,
    converter::converter::{Converter, get_custom_converter},
};
use anyhow::Result;
use tracing::{debug, info, trace, warn};

pub struct ConversionBlock {
    pub text: String,
    pub converter: Box<dyn Converter>,
}

pub struct Conversion;

impl Conversion {
    pub fn new() -> Self {
        info!("Creating new Conversion instance");
        Self {}
    }

    pub fn convert_text(&self, text: &str) -> Result<String> {
        info!("Converting text: {}", text);
        trace!("Text length: {}", text.len());
        let blocks = self.split_text(text)?;
        trace!("Number of blocks after splitting: {}", blocks.len());
        self.convert_blocks(blocks)
    }

    pub fn convert_blocks(&self, blocks: Vec<ConversionBlock>) -> Result<String> {
        debug!("Converting blocks");
        let mut result = String::new();
        for (index, block) in blocks.iter().enumerate() {
            trace!("Processing block {}/{}", index + 1, blocks.len());
            let converted = self.convert_block(&block)?;
            debug!(
                "Converted block - {}: {} -> {}",
                block.converter.name(),
                block.text,
                converted
            );
            result.push_str(&converted);
            trace!("Current result length: {}", result.len());
        }
        trace!("Final conversion result: {}", result);
        Ok(result)
    }

    pub fn convert_block(&self, block: &ConversionBlock) -> Result<String> {
        trace!("Converting block: {}", block.text);
        trace!("Using converter: {}", block.converter.name());
        if block.text.is_empty() {
            trace!("Empty block, returning default string");
            return Ok(String::default());
        }
        let result = block.converter.convert(&block.text);
        trace!("Conversion result: {:?}", result);
        result
    }

    pub fn split_text(&self, text: &str) -> Result<Vec<ConversionBlock>> {
        debug!("Splitting text: {}", text);
        let mut text = text.to_string();
        let mut blocks = Vec::new();
        let mut current_converter = 'r';

        let config = self.get_config();
        trace!(
            "Config command: {}, split: {}",
            config.command, config.split
        );

        if text.starts_with(&config.command) {
            trace!("Text starts with command");
            text = text.split_off(1);
            if !text.is_empty() {
                current_converter = text.chars().next().unwrap_or('n');
                text = text.split_off(1);
                trace!("Initial converter set to: {}", current_converter);
            }
        }

        for (i, command_splitted) in text.split(&config.command).enumerate() {
            trace!("Processing split {}", i);
            let mut command_splitted = command_splitted.to_string();
            if i != 0 {
                if !command_splitted.is_empty() {
                    current_converter = command_splitted.chars().next().unwrap_or('n');
                    command_splitted = command_splitted.split_off(1);
                    trace!("Converter changed to: {}", current_converter);
                }
            }

            for splitted in command_splitted.split(&config.split) {
                trace!(
                    "Creating ConversionBlock - text: {}, converter: {}",
                    splitted, current_converter
                );
                let converter = get_custom_converter(current_converter).unwrap_or_else(|| {
                    warn!(
                        "Failed to get custom converter for '{}', using default",
                        current_converter
                    );
                    get_custom_converter('n').unwrap()
                });
                blocks.push(ConversionBlock {
                    text: splitted.to_string(),
                    converter,
                });
            }
        }

        debug!("Split text into {} blocks", blocks.len());
        trace!(
            "Blocks: {:?}",
            blocks.iter().map(|b| &b.text).collect::<Vec<_>>()
        );
        Ok(blocks)
    }

    pub fn get_config(&self) -> Config {
        trace!("Getting config");
        let config = STATE.lock().unwrap().clone();
        trace!("Config retrieved: {:?}", config);
        config
    }
}
