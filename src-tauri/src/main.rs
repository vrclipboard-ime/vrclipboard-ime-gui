// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod azookey;
mod com;
mod config;
mod conversion;
mod converter;
mod dictionary;
mod felanguage;
mod handler;
mod tauri_emit_subscriber;
mod transform_rule;
mod tsf;
mod tsf_availability;
mod tsf_conversion;
mod vr;

use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::{Mutex, OnceLock, RwLock},
};

use azookey::server::AzookeyConversionServer;
use once_cell::sync::Lazy;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use clipboard_master::Master;
use com::Com;
use config::Config;
use dictionary::Dictionary;
use handler::ConversionHandler;
use tauri_emit_subscriber::TauriEmitSubscriber;
use tauri_plugin_updater::UpdaterExt;
use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[cfg(target_os = "windows")]
use tsf_availability::check_tsf_availability;

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
static SERVER_PROCESS: Lazy<Mutex<Option<std::process::Child>>> = Lazy::new(|| Mutex::new(None));

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
    #[cfg(target_os = "windows")]
    {
        match check_tsf_availability() {
            Ok(result) => {
                debug!("TSF availability check result: {}", result);
                Ok(result)
            }
            Err(e) => Err(format!("Failed to check TSF availability: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        debug!("TSF availability check is not supported on this OS");
        Ok(false)
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
                }
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

#[tauri::command]
async fn register_manifest() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        vr::create_vrmanifest().unwrap();
        vr::register_manifest_with_openvr().unwrap();
    }
    Ok(())
}

fn start_server_process() -> String {
    let exe_path = std::env::current_exe().unwrap();
    let server_path = exe_path.to_str().unwrap();

    let mut command = std::process::Command::new(server_path);
    command.arg("server");

    let mut child = command.stdout(Stdio::piped()).spawn().unwrap();

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let mut server_name = String::new();
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("$") {
            server_name = line[1..line.len() - 1].to_string();
            break;
        }
    }

    *SERVER_PROCESS.lock().unwrap() = Some(child);

    server_name
}

fn cleanup_server_process() {
    if let Some(mut child) = SERVER_PROCESS.lock().unwrap().take() {
        debug!("Terminating server process");
        let _ = child.kill();
        let _ = child.wait();
    }
}

static SERVER_NAME: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

struct ServerProcessGuard;

impl Drop for ServerProcessGuard {
    fn drop(&mut self) {
        cleanup_server_process();
    }
}

static _SERVER_GUARD: Lazy<ServerProcessGuard> = Lazy::new(|| ServerProcessGuard);

fn extract_dictionary() {
    let self_exe_path = PathBuf::from(SELF_EXE_PATH.read().unwrap().as_str());
    let zip_path = self_exe_path
        .parent()
        .unwrap()
        .join("AzooKeyDictionary.zip");
    let app_dirs = AppDirs::new(Some("vrclipboard-ime"), false).unwrap();
    let extract_path = app_dirs.config_dir.join("AzooKeyDictionary");
    if !extract_path.exists() {
        std::fs::create_dir_all(&extract_path).unwrap();
        let mut zip = zip::ZipArchive::new(std::fs::File::open(zip_path).unwrap()).unwrap();
        zip.extract(&extract_path).unwrap();
    }
}

pub static SELF_EXE_PATH: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::default()));

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    SELF_EXE_PATH.write().unwrap().push_str(&args[0]);

    if args.contains(&"server".to_string()) {
        let server = AzookeyConversionServer::new();
        let server_name = &server.server_name;
        println!("${}$", server_name);
        server.server_loop();
        return;
    }

    let server_name = start_server_process();
    SERVER_NAME.lock().unwrap().replace(server_name);

    Lazy::force(&_SERVER_GUARD);

    extract_dictionary();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            config: Mutex::new(Config::load().unwrap_or_else(|_| {
                Config::generate_default_config().expect("Failed to generate default config");
                Config::load().expect("Failed to load default config")
            })),
            dictionary: Mutex::new(Dictionary::load().unwrap_or_else(|_| {
                Dictionary::generate_default_dictionary()
                    .expect("Failed to generate default dictionary");
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
            check_update,
            register_manifest,
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
                let _ = update_handle
                    .updater()
                    .unwrap()
                    .check()
                    .await
                    .map_err(|e| tracing::error!("Failed to check for updates: {}", e));
            });

            std::thread::spawn(move || {
                #[cfg(target_os = "windows")]
                let _com = Com::new().unwrap();

                let conversion_handler = ConversionHandler::new(app_handle).unwrap();

                let mut master = Master::new(conversion_handler);

                master.run().unwrap();
            });

            Ok(())
        })
        .on_window_event(|_window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                cleanup_server_process();
            }
            tauri::WindowEvent::Destroyed => {
                cleanup_server_process();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    cleanup_server_process();
}
