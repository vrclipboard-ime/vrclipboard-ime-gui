use anyhow::Result;
use tracing::debug;
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{CoInitialize, CoUninitialize};

pub struct Com;

#[cfg(target_os = "windows")]
impl Drop for Com {
    fn drop(&mut self) {
        debug!("Dropping Com instance");
        unsafe {
            CoUninitialize();
            debug!("CoUninitialize called");
        };
    }
}

#[cfg(target_os = "windows")]
impl Com {
    pub fn new() -> Result<Self> {
        unsafe {
            let _ = CoInitialize(None);
        };
        Ok(Com)
    }
}

#[cfg(not(target_os = "windows"))]
impl Com {
    pub fn new() -> Result<Self> {
        unimplemented!("Com is only implemented for Windows");
    }
}
