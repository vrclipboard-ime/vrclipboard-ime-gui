use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use platform_dirs::AppDirs;
use serde::Serialize;
use serde_derive::Deserialize;
use tauri::State;
use tracing::{debug, error, info, trace};

use crate::AppState;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "semicolon")]
    pub prefix: String,
    #[serde(default = "slash")]
    pub split: String,
    #[serde(default = "semicolon")]
    pub command: String,
    #[serde(default = "bool_true")]
    pub ignore_prefix: bool,
    #[serde(default)]
    pub on_copy_mode: OnCopyMode,
    #[serde(default = "bool_true")]
    pub skip_url: bool,
    #[serde(default = "bool_true")]
    pub use_tsf_reconvert: bool,
    #[serde(default = "bool_true")]
    pub skip_on_out_of_vrc: bool,
    #[serde(default = "bool_false")]
    pub tsf_announce: bool,
    #[serde(default = "bool_false")]
    pub use_azookey_conversion: bool,
    #[serde(default = "bool_false")]
    pub azookey_announce: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: ";".to_string(),
            split: "/".to_string(),
            command: ";".to_string(),
            ignore_prefix: true,
            on_copy_mode: OnCopyMode::ReturnToChatbox,
            skip_url: true,
            use_tsf_reconvert: true,
            skip_on_out_of_vrc: true,
            tsf_announce: false,
            use_azookey_conversion: false,
            azookey_announce: false,
        }
    }
}

#[inline]
fn slash() -> String {
    String::from("/")
}
#[inline]
fn semicolon() -> String {
    String::from(";")
}
#[inline]
fn bool_true() -> bool {
    true
}
#[inline]
fn bool_false() -> bool {
    false
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OnCopyMode {
    ReturnToClipboard,
    ReturnToChatbox,
    SendDirectly,
}

impl Default for OnCopyMode {
    fn default() -> Self {
        Self::ReturnToChatbox
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        debug!("Loading config");
        std::fs::create_dir_all(Self::get_path()).unwrap();

        let config_path = Self::get_path().join("config.yaml");
        if !Path::new(&config_path).exists() {
            info!("Config file not found, generating default");
            Self::generate_default_config()?;
        }
        let mut file = File::open(&config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        trace!("Raw config contents: {}", contents);
        let mut config: Config = serde_yaml::from_str(&contents)?;
        if !config.tsf_announce {
            config.use_tsf_reconvert = true;
            config.tsf_announce = true;
        }
        if !config.azookey_announce {
            config.use_tsf_reconvert = false;
            config.use_azookey_conversion = true;
            config.azookey_announce = true;
        }
        debug!("Config loaded successfully");
        Ok(config)
    }

    pub fn save(&self, state: State<AppState>) -> Result<(), String> {
        debug!("Saving config");
        std::fs::create_dir_all(Self::get_path()).unwrap();

        let config_path = Self::get_path().join("config.yaml");
        let mut file = match File::create(&config_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create config file: {}", e);
                return Err(format!("Failed to create config file: {}", e));
            }
        };

        match serde_yaml::to_string(&self) {
            Ok(yaml) => {
                trace!("Config to be saved: {}", yaml);
                if let Err(e) = file.write_all(yaml.as_bytes()) {
                    error!("Failed to write config: {}", e);
                    return Err(format!("Failed to write config: {}", e));
                }
                let mut app_config = state.config.lock().unwrap();
                *app_config = self.clone();
                info!("Config saved successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to serialize config: {}", e);
                Err(format!("Failed to serialize config: {}", e))
            }
        }
    }

    pub fn generate_default_config() -> Result<()> {
        debug!("Generating default config");
        let config_path = Self::get_path().join("config.yaml");
        let mut file = File::create(&config_path)?;
        let default_config = Config::default();
        let yaml = serde_yaml::to_string(&default_config).unwrap();
        file.write_all(yaml.as_bytes())?;
        file.flush()?;
        info!("Default config generated successfully");
        Ok(())
    }

    pub fn get_path() -> PathBuf {
        let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false).unwrap();
        let app_data = app_dirs.config_dir;
        trace!("Config path: {:?}", app_data);
        app_data
    }
}
