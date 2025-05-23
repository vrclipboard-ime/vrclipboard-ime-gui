use std::{fs, path::PathBuf, process::Command};

use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::SELF_EXE_PATH;

const APP_KEY: &str = "dev.mii.vrclipboardime";

pub fn create_vrmanifest() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting VR manifest file creation");

    let self_exe_path = PathBuf::from(SELF_EXE_PATH.read().unwrap().as_str());
    debug!("Executable path: {}", self_exe_path.display());

    let manifest = json!({
        "source": "builtin",
        "applications": [{
            "app_key": APP_KEY,
            "launch_type": "binary",
            "binary_path_windows": self_exe_path.to_string_lossy(),
            "is_dashboard_overlay": true,
            "strings": {
                "en_us": {
                    "name": "VRClipboard-IME",
                    "description": "VRClipboard-IME"
                },
                "ja_jp": {
                    "name": "VRClipboard-IME",
                    "description": "VRClipboard-IME"
                }
            }
        }]
    });

    debug!(
        "Manifest content to be created:\n{}",
        serde_json::to_string_pretty(&manifest)?
    );

    let exe_dir = self_exe_path
        .parent()
        .ok_or("Failed to get executable directory")?
        .to_path_buf();

    let manifest_path = exe_dir.join("vrclipboard-ime.vrmanifest");
    debug!("Manifest file path: {}", manifest_path.display());

    let manifest_content = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, &manifest_content)?;
    info!(
        "Successfully created VR manifest file: {}",
        manifest_path.display()
    );
    debug!(
        "Write completed - File size: {} bytes",
        manifest_content.len()
    );

    Ok(())
}

pub fn register_manifest_with_openvr() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting OpenVR manifest registration using proper method");

    let self_exe_path = PathBuf::from(SELF_EXE_PATH.read().unwrap().as_str());
    let exe_dir = self_exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;
    let manifest_path = exe_dir.join("vrclipboard-ime.vrmanifest");

    info!("Registering manifest file: {}", manifest_path.display());

    let steam_config_dir = get_steam_config_directory()?;
    let steam_dir = steam_config_dir
        .parent()
        .ok_or("Failed to get Steam directory")?;

    let vrcmd_path = steam_dir
        .join("steamapps")
        .join("common")
        .join("SteamVR")
        .join("bin")
        .join("win64")
        .join("vrcmd.exe");

    if vrcmd_path.exists() {
        info!("Using vrcmd.exe to register manifest");
        debug!("vrcmd.exe path: {}", vrcmd_path.display());

        let output = Command::new(&vrcmd_path)
            .args(&["--appmanifest", manifest_path.to_string_lossy().as_ref()])
            .output()?;

        debug!("vrcmd exit status: {}", output.status);

        if output.status.success() {
            info!("Successfully registered manifest with vrcmd");
        }
    } else {
        return Err("vrcmd.exe not found".into());
    }

    info!("OpenVR manifest registration completed");
    Ok(())
}

fn get_steam_config_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    info!("Searching for Steam configuration directory");

    let possible_paths = vec![PathBuf::from(r"C:\Program Files (x86)\Steam\config")];

    debug!("Attempting to get Steam path from registry");
    if let Ok(steam_path) = get_steam_path_from_registry() {
        info!("Found Steam path in registry: {}", steam_path.display());
        let config_path = steam_path.join("config");
        if config_path.exists() {
            info!(
                "Found Steam configuration directory: {}",
                config_path.display()
            );
            return Ok(config_path);
        } else {
            warn!(
                "Configuration directory from registry does not exist: {}",
                config_path.display()
            );
        }
    } else {
        debug!("Failed to get Steam path from registry");
    }

    debug!("Searching in standard paths");
    for path in &possible_paths {
        debug!("Checking path: {}", path.display());
        if path.exists() {
            info!("Found Steam configuration directory: {}", path.display());
            return Ok(path.clone());
        }
    }

    error!(
        "Steam configuration directory not found. Checked paths: {:?}",
        possible_paths
    );
    Err("Steam configuration directory not found".into())
}

fn get_steam_path_from_registry() -> Result<PathBuf, Box<dyn std::error::Error>> {
    debug!("Reading Steam path from registry");

    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        use std::ptr;
        use winapi::um::winnt::*;
        use winapi::um::winreg::*;

        let mut hkey = ptr::null_mut();
        let registry_path = "SOFTWARE\\WOW6432Node\\Valve\\Steam";
        debug!("Opening registry key: {}", registry_path);

        let result = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\WOW6432Node\\Valve\\Steam\0"
                .encode_utf16()
                .collect::<Vec<u16>>()
                .as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result == 0 {
            debug!("Successfully opened registry key");
            let mut buffer = vec![0u16; 512];
            let mut buffer_size = (buffer.len() * 2) as u32;

            let result = RegQueryValueExW(
                hkey,
                "InstallPath\0"
                    .encode_utf16()
                    .collect::<Vec<u16>>()
                    .as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8,
                &mut buffer_size,
            );

            RegCloseKey(hkey);

            if result == 0 {
                let len = (buffer_size / 2) as usize;
                if len > 0 && buffer[len - 1] == 0 {
                    buffer.truncate(len - 1);
                }
                let path_string = OsString::from_wide(&buffer)
                    .into_string()
                    .map_err(|_| "Failed to convert registry path to UTF-8")?;
                let steam_path = PathBuf::from(path_string);
                debug!(
                    "Retrieved Steam path from registry: {}",
                    steam_path.display()
                );
                return Ok(steam_path);
            } else {
                warn!(
                    "Failed to read InstallPath value. Registry error code: {}",
                    result
                );
            }
        } else {
            warn!("Failed to open registry key. Error code: {}", result);
        }
    }

    Err("Failed to get Steam path from registry".into())
}
