use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info, trace};

use crate::{
    config::Config,
    converter::converter::{get_custom_converter, Converter},
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConversionMethod {
    Replace,
    None,
    Converter(char),
}

impl Default for ConversionMethod {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DictionaryEntry {
    pub input: String,
    pub method: ConversionMethod,
    pub output: Option<String>,
    pub use_regex: bool,
    pub priority: i32,
}

impl Default for DictionaryEntry {
    fn default() -> Self {
        Self {
            input: String::new(),
            method: ConversionMethod::None,
            output: None,
            use_regex: false,
            priority: 0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dictionary {
    pub entries: Vec<DictionaryEntry>,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Dictionary {
    pub fn load() -> Result<Dictionary> {
        debug!("Loading dictionary");
        std::fs::create_dir_all(Config::get_path())?;

        let dict_path = Self::get_path();
        if !dict_path.exists() {
            info!("Dictionary file not found, generating default");
            Self::generate_default_dictionary()?;
        }
        let mut file = File::open(&dict_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        trace!("Raw dictionary contents: {}", contents);
        let dictionary: Dictionary = serde_yaml::from_str(&contents)?;
        debug!(
            "Dictionary loaded successfully with {} entries",
            dictionary.entries.len()
        );
        Ok(dictionary)
    }

    pub fn save(&self, state: State<AppState>) -> Result<(), String> {
        debug!("Saving dictionary");
        std::fs::create_dir_all(Config::get_path()).unwrap();

        let dict_path = Self::get_path();
        let mut file = match File::create(&dict_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create dictionary file: {}", e);
                return Err(format!("Failed to create dictionary file: {}", e));
            }
        };

        match serde_yaml::to_string(&self) {
            Ok(yaml) => {
                trace!("Dictionary to be saved: {}", yaml);
                if let Err(e) = file.write_all(yaml.as_bytes()) {
                    error!("Failed to write dictionary: {}", e);
                    return Err(format!("Failed to write dictionary: {}", e));
                }
                let mut app_dictionary = state.dictionary.lock().unwrap();
                *app_dictionary = self.clone();
                info!("Dictionary saved successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to serialize dictionary: {}", e);
                Err(format!("Failed to serialize dictionary: {}", e))
            }
        }
    }

    pub fn generate_default_dictionary() -> Result<()> {
        debug!("Generating default dictionary");
        let dict_path = Self::get_path();
        let mut file = File::create(&dict_path)?;
        let default_dict = Dictionary::default();
        let yaml = serde_yaml::to_string(&default_dict).unwrap();
        file.write_all(yaml.as_bytes())?;
        file.flush()?;
        info!("Default dictionary generated successfully");
        Ok(())
    }

    pub fn get_path() -> PathBuf {
        let path = Config::get_path().join("dictionary.yaml");
        trace!("Dictionary path: {:?}", path);
        path
    }

    pub fn apply_conversion(&self, text: &str) -> Result<String> {
        debug!("Applying dictionary conversions to: {}", text);

        todo!()
    }
}
