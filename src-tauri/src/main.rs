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

use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use clipboard_master::Master;
use com::Com;
use config::Config;
use handler::ConversionHandler;
use tauri_emit_subscriber::TauriEmitSubscriber;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Log {
    pub time: String,
    pub original: String,
    pub converted: String,
}

struct AppState {
    config: Mutex<Config>,
}

static STATE: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::load().unwrap()));

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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            config: Mutex::new(Config::load().unwrap_or_else(|_| {
                Config::generate_default_config().expect("Failed to generate default config");
                Config::load().expect("Failed to load default config")
            })),
        })
        .invoke_handler(tauri::generate_handler![load_settings, save_settings])
        .setup(|app| {
            let _span = tracing::span!(tracing::Level::INFO, "main");
            app.manage(STATE.lock().unwrap().clone());
            let app_handle = app.app_handle().clone();

            let registry = tracing_subscriber::registry().with(TauriEmitSubscriber {
                app_handle: app_handle.clone(),
            });
            registry.init();

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
