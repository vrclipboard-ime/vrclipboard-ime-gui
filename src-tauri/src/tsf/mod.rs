use anyhow::Result;
use tracing::{debug, error};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_SETTHREADLOCALINPUTSETTINGS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

pub mod function_provider;
pub mod input_processor_profile_mgr;
pub mod search_candidate_provider;
pub mod thread_mgr;

#[cfg(target_os = "windows")]
pub fn set_thread_local_input_settings(thread_local_input_settings: bool) -> Result<()> {
    debug!(
        "Setting thread local input settings to: {}",
        thread_local_input_settings
    );
    let mut result = thread_local_input_settings;
    match unsafe {
        SystemParametersInfoW(
            SPI_SETTHREADLOCALINPUTSETTINGS,
            0,
            Some(&mut result as *mut _ as *const _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    } {
        Ok(_) => {
            debug!("Successfully set thread local input settings");
            Ok(())
        }
        Err(e) => {
            error!("Failed to set thread local input settings: {:?}", e);
            Err(e.into())
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_thread_local_input_settings(_thread_local_input_settings: bool) -> Result<()> {
    debug!("set_thread_local_input_settings is not implemented on this OS");
    Ok(())
}
