// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod com;
mod config;
mod conversion;
mod converter;
mod felanguage;
mod handler;
mod transform_rule;
mod tsf;
mod tsf_conversion;
mod tauri_emit_subscriber;
mod tsf_availability;
mod dictionary;

use std::sync::{Mutex, OnceLock};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use clipboard_master::Master;
use com::Com;
use config::Config;
use handler::ConversionHandler;
use tauri_emit_subscriber::TauriEmitSubscriber;
use tauri_plugin_updater::UpdaterExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tsf_availability::check_tsf_availability;
use tracing::{debug, error};
use dictionary::Dictionary;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Log {
    pub time: String,
    pub original: String,
    pub converted: String,
}

struct AppState {
    config: Mutex<Config>,
    dictionary: Mutex<Dictionary>,
}

static STATE: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::load().unwrap()));
static DICTIONARY: Lazy<Mutex<Dictionary>> = Lazy::new(|| Mutex::new(Dictionary::load().unwrap()));
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[tauri::command]
fn load_settings(state: State<AppState>) -> Result<Config, String> {
    match Config::load() {
        Ok(config) => {
            let mut app_config = state.config.lock().unwrap();
            *app_config = config.clone();
            Ok(config)
        }
        Err(e) => Err(format!("Failed to load settings: {}", e)),
    }
}

#[tauri::command]
fn save_settings(config: Config, state: State<AppState>) -> Result<(), String> {
    *STATE.lock().unwrap() = config.clone();
    config.save(state)
}

#[tauri::command]
fn check_tsf_availability_command() -> Result<bool, String> {
    debug!("Checking TSF availability");
    match check_tsf_availability() {
        Ok(result) => {
            debug!("TSF availability check result: {}", result);
            Ok(result)
        },
        Err(e) => Err(format!("Failed to check TSF availability: {}", e)),
    }
}

#[tauri::command]
fn load_dictionary(state: State<AppState>) -> Result<Dictionary, String> {
    match Dictionary::load() {
        Ok(dictionary) => {
            let mut app_dictionary = state.dictionary.lock().unwrap();
            *app_dictionary = dictionary.clone();
            Ok(dictionary)
        }
        Err(e) => Err(format!("Failed to load dictionary: {}", e)),
    }
}

#[tauri::command]
fn save_dictionary(dictionary: Dictionary, state: State<AppState>) -> Result<(), String> {
    *DICTIONARY.lock().unwrap() = dictionary.clone();
    dictionary.save(state).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_ms_settings_regionlanguage_jpnime() -> Result<(), String> {
    let _ = std::process::Command::new("cmd")
        .args(&["/C", "start", "ms-settings:regionlanguage-jpnime"])
        .output()
        .map_err(|e| format!("Failed to open MS Settings: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn check_update() -> Result<bool, String> {
    if let Some(app_handle) = APP_HANDLE.get() {
        match app_handle.updater() {
            Ok(updater) => match updater.check().await {
                Ok(Some(_)) => Ok(true),
                Ok(None) => Ok(false),
                Err(e) => {
                    error!("Failed to check for updates: {}", e);
                    Err(format!("Failed to check for updates: {}", e))
                },
            },
            Err(e) => {
                error!("Updater not available: {}", e);
                Err("Updater not available".to_string())
            }
        }
    } else {
        error!("App handle not set");
        Err("App handle not set".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            config: Mutex::new(Config::load().unwrap_or_else(|_| {
                Config::generate_default_config().expect("Failed to generate default config");
                Config::load().expect("Failed to load default config")
            })),
            dictionary: Mutex::new(Dictionary::load().unwrap_or_else(|_| {
                Dictionary::generate_default_dictionary().expect("Failed to generate default dictionary");
                Dictionary::load().expect("Failed to load default dictionary")
            })),
        })
        .invoke_handler(tauri::generate_handler![
            load_settings, 
            save_settings, 
            check_tsf_availability_command, 
            open_ms_settings_regionlanguage_jpnime, 
            load_dictionary, 
            save_dictionary,
            check_update
        ])
        .setup(|app| {
            APP_HANDLE.set(app.app_handle().to_owned()).unwrap();

            let _span = tracing::span!(tracing::Level::INFO, "main");
            app.manage(STATE.lock().unwrap().clone());
            app.manage(DICTIONARY.lock().unwrap().clone());
            let app_handle = app.app_handle().clone();

            let registry = tracing_subscriber::registry().with(TauriEmitSubscriber {
                app_handle: app_handle.clone(),
            });
            registry.init();

            let update_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let _ = update_handle.updater()
                    .unwrap()
                    .check()
                    .await
                    .map_err(|e| tracing::error!("Failed to check for updates: {}", e));
            });

            std::thread::spawn(move || {
                let _com = Com::new().unwrap();

                let conversion_handler = ConversionHandler::new(app_handle).unwrap();

                let mut master = Master::new(conversion_handler);

                master.run().unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}