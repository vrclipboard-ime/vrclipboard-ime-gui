// Prevent an additional console window in Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
#[cfg(feature = "azookey")]
mod azookey;
mod com;
mod config;
mod conversion;
mod converter;
mod dictionary;
mod events;
mod felanguage;
mod handler;
mod icons;
mod tsf;
mod tsf_availability;
mod tsf_conversion;
mod ui;
mod ui_log_subscriber;
mod vr;

use std::{path::PathBuf, sync::Mutex};

use clipboard_master::Master;
use com::Com;
use config::Config;
use dictionary::Dictionary;
use events::AppEvent;
use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, px, size};
use handler::ConversionHandler;
use once_cell::sync::Lazy;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ui_log_subscriber::UiLogSubscriber;

pub static STATE: Lazy<Mutex<Config>> = Lazy::new(|| {
    Mutex::new(Config::load().unwrap_or_else(|_| {
        Config::generate_default_config().expect("failed to generate default config");
        Config::load().expect("failed to load default config")
    }))
});

pub static DICTIONARY: Lazy<Mutex<Dictionary>> = Lazy::new(|| {
    Mutex::new(Dictionary::load().unwrap_or_else(|_| {
        Dictionary::generate_default_dictionary().expect("failed to generate default dictionary");
        Dictionary::load().expect("failed to load default dictionary")
    }))
});

fn resource_dir() -> PathBuf {
    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
        && directory.join("azookey-native").is_dir()
    {
        return directory.to_path_buf();
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
}

fn start_clipboard_monitor(sender: flume::Sender<AppEvent>) {
    let resources = resource_dir();
    std::thread::Builder::new()
        .name("clipboard-monitor".into())
        .spawn(move || {
            #[cfg(target_os = "windows")]
            let _com = match Com::new() {
                Ok(com) => Some(com),
                Err(error) => {
                    error!(%error, "COM initialization failed");
                    None
                }
            };

            let mut conversion_handler = match ConversionHandler::new(sender, resources) {
                Ok(handler) => handler,
                Err(error) => {
                    error!(%error, "conversion handler initialization failed");
                    return;
                }
            };

            #[cfg(feature = "azookey")]
            if let Err(error) = conversion_handler.warm_up_azookey() {
                error!(%error, "AzooKey conversion warm-up failed");
            }

            match Master::new(conversion_handler) {
                Ok(mut master) => {
                    if let Err(error) = master.run() {
                        error!(%error, "clipboard monitor stopped");
                    }
                }
                Err(error) => error!(%error, "clipboard monitor creation failed"),
            }
        })
        .expect("failed to start clipboard monitor thread");
}

fn main() {
    let (event_sender, event_receiver) = flume::unbounded();

    let _ = tracing_subscriber::registry()
        .with(UiLogSubscriber {
            event_sender: event_sender.clone(),
        })
        .try_init();

    // Force lazy state initialization before the background monitor starts.
    drop(STATE.lock().unwrap());
    drop(DICTIONARY.lock().unwrap());
    start_clipboard_monitor(event_sender);

    Application::new()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            gpui_component::Theme::change(gpui_component::ThemeMode::Light, None, cx);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();

            let bounds = Bounds::centered(None, size(px(800.), px(640.)), cx);
            let receiver = event_receiver.clone();
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui_component::TitleBar::title_bar_options()),
                    window_min_size: Some(size(px(640.), px(480.))),
                    app_id: Some("dev.mii.vrclipboard-ime".into()),
                    ..Default::default()
                },
                move |window, cx| ui::root_view(receiver, window, cx),
            )
            .expect("failed to open GPUI window");

            cx.activate(true);
        });
}
