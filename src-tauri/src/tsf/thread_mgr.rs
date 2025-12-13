use anyhow::Result;
use tracing::{debug, error, info};
#[cfg(target_os = "windows")]
use windows::Win32::{
    System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
    UI::TextServices::{CLSID_TF_ThreadMgr, ITfFunctionProvider, ITfThreadMgr2},
};

pub struct ThreadMgr {
    #[cfg(target_os = "windows")]
    pub thread_mgr: ITfThreadMgr2,
}

#[cfg(target_os = "windows")]
impl ThreadMgr {
    pub fn new() -> Result<Self> {
        debug!("Creating new ThreadMgr");
        let thread_mgr =
            unsafe { CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER)? };
        info!("ThreadMgr created successfully");
        Ok(ThreadMgr { thread_mgr })
    }

    pub fn activate_ex(&self, flags: u32) -> Result<u32> {
        debug!("Activating ThreadMgr with flags: {}", flags);
        let mut client_id = 0;
        unsafe {
            self.thread_mgr
                .ActivateEx(&mut client_id as *mut _ as *const _ as *mut _, flags)?
        };
        info!("ThreadMgr activated with client_id: {}", client_id);
        Ok(client_id)
    }

    pub fn get_function_provider(&self, clsid: &windows_core::GUID) -> Result<ITfFunctionProvider> {
        debug!("Getting function provider for CLSID: {:?}", clsid);
        match unsafe { self.thread_mgr.GetFunctionProvider(clsid) } {
            Ok(provider) => {
                info!("Function provider obtained successfully");
                Ok(provider)
            }
            Err(e) => {
                error!("Failed to get function provider: {:?}", e);
                Err(e.into())
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl ThreadMgr {
    pub fn new() -> Result<Self> {
        debug!("ThreadMgr is not implemented on this OS");
        Ok(ThreadMgr {})
    }
}
