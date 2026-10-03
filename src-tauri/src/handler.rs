use std::{net::UdpSocket, path::PathBuf};

#[cfg(feature = "azookey")]
use crate::azookey::{azookey_conversion::AzookeyConversion, client::AzookeyConversionClient};
#[cfg(target_os = "windows")]
use crate::chatbox_input::{ChatboxTarget, is_vrchat_window};
#[cfg(target_os = "windows")]
use crate::tsf_conversion::TsfConversion;
use crate::{
    STATE,
    config::{Config, OnCopyMode},
    conversion::Conversion,
    events::{AppEvent, ConversionLog},
};
use anyhow::Result;
use chrono::Local;
use clipboard::{ClipboardContext, ClipboardProvider};
use clipboard_master::{CallbackResult, ClipboardHandler};
use flume::Sender;
use regex::Regex;
use rosc::{OscMessage, OscPacket, OscType, encoder};
use tracing::{error, info, warn};
#[cfg(target_os = "windows")]
use windows::Win32::System::DataExchange::{GetClipboardOwner, GetClipboardSequenceNumber};

#[cfg(target_os = "windows")]
#[derive(Default)]
struct ClipboardUpdates {
    last_sequence: Option<u32>,
}

#[cfg(target_os = "windows")]
impl ClipboardUpdates {
    fn accept(&mut self, sequence: u32) -> bool {
        if sequence == 0 || self.last_sequence == Some(sequence) {
            return false;
        }
        self.last_sequence = Some(sequence);
        true
    }

    fn record_write(&mut self, sequence: u32) {
        self.last_sequence = Some(sequence);
    }
}

pub struct ConversionHandler {
    event_sender: Sender<AppEvent>,
    resource_dir: PathBuf,
    conversion: Conversion,
    #[cfg(target_os = "windows")]
    tsf_conversion: Option<TsfConversion>,
    #[cfg(feature = "azookey")]
    azookey_conversion: Option<AzookeyConversion>,
    clipboard_ctx: ClipboardContext,
    last_copy: String,
    #[cfg(target_os = "windows")]
    clipboard_updates: ClipboardUpdates,
    #[cfg(target_os = "windows")]
    chatbox_target: Option<ChatboxTarget>,
}

impl ConversionHandler {
    pub fn new(event_sender: Sender<AppEvent>, resource_dir: PathBuf) -> Result<Self> {
        let conversion = Conversion::new();
        #[cfg(target_os = "windows")]
        let tsf_conversion = None;
        #[cfg(feature = "azookey")]
        let azookey_conversion = None;
        let clipboard_ctx = ClipboardProvider::new().unwrap();

        info!("ConversionHandler created");
        Ok(Self {
            event_sender,
            resource_dir,
            conversion,
            #[cfg(target_os = "windows")]
            tsf_conversion,
            #[cfg(feature = "azookey")]
            azookey_conversion,
            clipboard_ctx,
            last_copy: String::new(),
            #[cfg(target_os = "windows")]
            clipboard_updates: ClipboardUpdates::default(),
            #[cfg(target_os = "windows")]
            chatbox_target: None,
        })
    }

    pub fn get_config(&self) -> Config {
        STATE.lock().unwrap().clone()
    }

    #[cfg(feature = "azookey")]
    pub fn warm_up_azookey(&mut self) -> Result<()> {
        let config = self.get_config();
        if !config.use_azookey_conversion
            || config.azookey_backend != crate::config::AzookeyBackend::Vulkan
        {
            return Ok(());
        }

        info!(backend = ?config.azookey_backend, "Warming up Azookey conversion");
        self.ensure_azookey_conversion(config.azookey_backend)?
            .warm_up()?;
        info!(backend = ?config.azookey_backend, "Azookey conversion warm-up completed");
        Ok(())
    }

    #[cfg(feature = "azookey")]
    fn ensure_azookey_conversion(
        &mut self,
        backend: crate::config::AzookeyBackend,
    ) -> Result<&mut AzookeyConversion> {
        let requested_backend = match backend {
            crate::config::AzookeyBackend::Cpu => azookey_kkc::Backend::Cpu,
            crate::config::AzookeyBackend::Vulkan => azookey_kkc::Backend::Vulkan,
        };
        let needs_initialization = match self.azookey_conversion.as_ref() {
            Some(conversion) => conversion.backend() != requested_backend,
            None => true,
        };
        if needs_initialization {
            let client = AzookeyConversionClient::new(&self.resource_dir, backend)?;
            self.azookey_conversion = Some(AzookeyConversion::new(client));
            info!(?requested_backend, "Azookey conversion created");
        }
        self.azookey_conversion
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Azookey conversion was not initialized"))
    }
}

impl ConversionHandler {
    #[cfg(target_os = "windows")]
    fn clipboard_has_non_vrc_owner(&self) -> bool {
        let owner = unsafe { GetClipboardOwner() };
        owner.0 != 0 && !is_vrchat_window(owner)
    }

    #[cfg(not(target_os = "windows"))]
    fn clipboard_has_non_vrc_owner(&self) -> bool {
        false
    }

    #[cfg(feature = "azookey")]
    fn azookey_conversion(&mut self, contents: &str, config: &Config) -> Result<()> {
        if contents.chars().count() > 140 {
            info!("Content exceeds 140 characters, skipping Azookey conversion");
            return Ok(());
        }
        if contents.is_empty() {
            info!("Empty content, skipping Azookey conversion");
            return Ok(());
        }
        if config.skip_url
            && Regex::new(r"(http://|https://){1}[\w\.\-/:\#\?=\&;%\~\+]+")
                .unwrap()
                .is_match(&contents)
        {
            info!("URL detected, skipping Azookey conversion");
            return Ok(());
        }

        let converted = self
            .ensure_azookey_conversion(config.azookey_backend)?
            .convert(contents)?;

        info!("Azookey conversion: {} -> {}", contents, converted);

        self.return_conversion(contents.to_string(), converted, config);

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn tsf_conversion(&mut self, contents: &str, config: &Config) -> Result<()> {
        if contents.chars().count() > 140 {
            info!("Content exceeds 140 characters, skipping TSF conversion");
            return Ok(());
        }
        if config.skip_url
            && Regex::new(r"(http://|https://){1}[\w\.\-/:\#\?=\&;%\~\+]+")
                .unwrap()
                .is_match(&contents)
        {
            info!("URL detected, skipping TSF conversion");
            return Ok(());
        }

        if self.tsf_conversion.is_none() {
            self.tsf_conversion = Some(TsfConversion::new());
            info!("TSF conversion created");
        }

        let tsf_conversion = self.tsf_conversion.as_mut().unwrap();

        let converted = tsf_conversion.convert(contents)?;

        info!("TSF conversion: {} -> {}", contents, converted);

        self.return_conversion(contents.to_string(), converted, config);

        Ok(())
    }

    fn write_conversion_to_clipboard(&mut self, converted: &str) -> Result<()> {
        for attempt in 0..5 {
            match self.clipboard_ctx.set_contents(converted.to_owned()) {
                Ok(()) => {
                    // Suppress this update, not future copies of the same text.
                    #[cfg(target_os = "windows")]
                    self.clipboard_updates
                        .record_write(unsafe { GetClipboardSequenceNumber() });
                    self.last_copy = converted.to_owned();
                    return Ok(());
                }
                Err(error) if attempt == 4 => {
                    return Err(anyhow::anyhow!("Failed to set clipboard contents: {error}"));
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(10)),
            }
        }
        unreachable!()
    }

    #[cfg(target_os = "windows")]
    fn paste_conversion_to_chatbox(&mut self, converted: &str) -> Result<()> {
        if let Some(target) = self.chatbox_target.as_ref() {
            target.wait_for_keys_released()?;
        }
        anyhow::ensure!(
            self.clipboard_ctx
                .get_contents()
                .map_err(|error| anyhow::anyhow!("Cannot read clipboard: {error}"))?
                == self.last_copy,
            "Clipboard changed during conversion; automatic paste cancelled"
        );
        anyhow::ensure!(
            self.clipboard_updates.last_sequence == Some(unsafe { GetClipboardSequenceNumber() }),
            "Clipboard was copied again during conversion; automatic paste cancelled"
        );
        self.write_conversion_to_clipboard(converted)?;
        let target = self.chatbox_target.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No unique VRChat window found; result is available in clipboard")
        })?;
        target.select_all_and_paste()?;
        info!(
            "Ctrl+A / Ctrl+V dispatched to VRChat via {}",
            target.method_name()
        );
        Ok(())
    }

    fn return_conversion(&mut self, parsed_contents: String, converted: String, config: &Config) {
        match config.on_copy_mode {
            OnCopyMode::ReturnToClipboard => {
                if let Err(error) = self.write_conversion_to_clipboard(&converted) {
                    error!("Failed to return conversion to clipboard: {error}");
                } else {
                    info!("Conversion returned to clipboard");
                }
            }
            OnCopyMode::ReturnToChatbox => {
                #[cfg(target_os = "windows")]
                if let Err(error) = self.paste_conversion_to_chatbox(&converted) {
                    warn!("Automatic chatbox paste cancelled or failed: {error}");
                }
                #[cfg(not(target_os = "windows"))]
                if let Err(error) = self.write_conversion_to_clipboard(&converted) {
                    error!("Failed to return conversion to clipboard: {error}");
                } else {
                    warn!(
                        "Automatic chatbox paste is only available on Windows; result is in clipboard"
                    );
                }
            }
            OnCopyMode::SendDirectly => {
                let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
                let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: "/chatbox/input".to_string(),
                    args: vec![
                        OscType::String(converted.clone()),
                        OscType::Bool(true),
                        OscType::Bool(true),
                    ],
                }))
                .unwrap();

                if let Err(e) = sock.send_to(&msg_buf, "127.0.0.1:9000") {
                    error!("Failed to send UDP packet: {}", e);
                } else {
                    info!("Conversion sent directly");
                }
            }
        }

        let datetime = Local::now();
        let _ = self.event_sender.send(AppEvent::Conversion(ConversionLog {
            time: datetime.format("%Y %m/%d %H:%M:%S").to_string(),
            original: parsed_contents,
            converted,
        }));
    }
}

impl ClipboardHandler for ConversionHandler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let config = self.get_config();
        if config.skip_on_out_of_vrc && self.clipboard_has_non_vrc_owner() {
            info!("Clipboard owner is outside VRChat, skipping conversion");
            return CallbackResult::Next;
        }

        if let Ok(mut contents) = self.clipboard_ctx.get_contents() {
            #[cfg(target_os = "windows")]
            if !self
                .clipboard_updates
                .accept(unsafe { GetClipboardSequenceNumber() })
            {
                return CallbackResult::Next;
            }
            #[cfg(not(target_os = "windows"))]
            if self.last_copy == contents {
                return CallbackResult::Next;
            }
            self.last_copy = contents.clone();
            #[cfg(target_os = "windows")]
            {
                self.chatbox_target = if matches!(config.on_copy_mode, OnCopyMode::ReturnToChatbox)
                {
                    match ChatboxTarget::capture() {
                        Ok(target) => Some(target),
                        Err(error) => {
                            warn!("Automatic chatbox paste unavailable: {error}");
                            None
                        }
                    }
                } else {
                    None
                };
            }
            info!("Clipboard changed: {}", contents);
            contents = contents
                .chars()
                .take_while(|&c| c != '\u{0000}')
                .collect::<String>();

            #[cfg(feature = "azookey")]
            if config.use_azookey_conversion {
                if let Err(e) = self.azookey_conversion(&contents, &config) {
                    error!("Azookey conversion failed: {}", e);
                }
                return CallbackResult::Next;
            }
            #[cfg(not(feature = "azookey"))]
            if config.use_azookey_conversion {
                return CallbackResult::Next;
            }

            if config.use_tsf_reconvert {
                #[cfg(target_os = "windows")]
                if let Err(e) = self.tsf_conversion(&contents, &config) {
                    error!("TSF conversion failed: {}", e);
                }
                return CallbackResult::Next;
            }

            if contents.starts_with(&config.prefix) || config.ignore_prefix {
                if config.skip_url
                    && Regex::new(r"(http://|https://){1}[\w\.\-/:\#\?=\&;%\~\+]+")
                        .unwrap()
                        .is_match(&contents)
                {
                    info!("URL detected, skipping conversion");
                    return CallbackResult::Next;
                }

                let parsed_contents = if config.ignore_prefix {
                    contents
                } else {
                    contents.split_off(1)
                };
                let converted = match self.conversion.convert_text(&parsed_contents) {
                    Ok(converted) => converted,
                    Err(err) => {
                        error!("Conversion error: {:?}", err);
                        format!("Error: {:?}", err)
                    }
                };

                info!("Conversion: {} -> {}", parsed_contents, converted);

                self.return_conversion(parsed_contents, converted, &config);
            }
        }
        CallbackResult::Next
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::ClipboardUpdates;

    #[test]
    fn recopied_conversion_result_is_processed_but_own_notification_is_ignored() {
        let mut updates = ClipboardUpdates::default();
        assert!(updates.accept(10)); // User copies original text.
        updates.record_write(11); // App writes the conversion result.
        assert!(!updates.accept(11)); // Its own notification must not reconvert.
        assert!(updates.accept(12)); // User copies that same result: reconvert.
        assert!(!updates.accept(12)); // Duplicate notification for the same copy.
        updates.record_write(13); // Reconversion writes its result.
        assert!(!updates.accept(13));
        assert!(updates.accept(14)); // Another explicit copy must still work.
    }
}
