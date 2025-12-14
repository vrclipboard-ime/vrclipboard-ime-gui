use std::ptr;

use anyhow::Result;
use tracing::{debug, error, info, trace};
#[cfg(target_os = "windows")]
use windows::{
    core::{w, PCWSTR},
    Win32::{
        System::Com::{CLSIDFromProgID, CoCreateInstance, CLSCTX_ALL},
        UI::Input::Ime::IFELanguage,
    },
};

pub struct FElanguage {
    #[cfg(target_os = "windows")]
    ife: IFELanguage,
}

impl Drop for FElanguage {
    fn drop(&mut self) {
        debug!("Dropping FElanguage instance");
        #[cfg(target_os = "windows")]
        if let Err(e) = unsafe { self.ife.Close() } {
            error!("Error closing IFELanguage: {:?}", e);
        }
    }
}

impl FElanguage {
    #[cfg(target_os = "windows")]
    pub fn new() -> Result<Self> {
        info!("Creating new FElanguage instance");
        let clsid = unsafe {
            trace!("Getting CLSID for MSIME.Japan");
            CLSIDFromProgID(w!("MSIME.Japan"))?
        };
        let ife: IFELanguage = unsafe {
            trace!("Creating IFELanguage instance");
            CoCreateInstance(&clsid, None, CLSCTX_ALL)?
        };
        unsafe {
            trace!("Opening IFELanguage");
            ife.Open()?
        };
        debug!("FElanguage instance created successfully");
        Ok(FElanguage { ife })
    }

    #[cfg(target_os = "windows")]
    pub fn j_morph_result(&self, input: &str, request: u32, mode: u32) -> Result<String> {
        debug!(
            "Calling j_morph_result with input: {}, request: {}, mode: {}",
            input, request, mode
        );
        let input_utf16: Vec<u16> = input.encode_utf16().chain(Some(0)).collect();
        let input_len = input_utf16.len();
        let input_pcwstr = PCWSTR::from_raw(input_utf16.as_ptr());

        let mut result_ptr = ptr::null_mut();
        unsafe {
            trace!("Calling GetJMorphResult");
            self.ife.GetJMorphResult(
                request,
                mode,
                input_len as _,
                input_pcwstr,
                ptr::null_mut(),
                &mut result_ptr,
            )?;
        }

        if result_ptr.is_null() {
            error!("GetJMorphResult returned null pointer");
            return Err(anyhow::anyhow!("GetJMorphResult returned null pointer"));
        }

        let result_struct = unsafe { &*result_ptr };
        let output_bstr_ptr = result_struct.pwchOutput;
        let output_len = result_struct.cchOutput as usize;

        if output_bstr_ptr.is_null() {
            error!("Output BSTR pointer is null");
            return Err(anyhow::anyhow!("Output BSTR pointer is null"));
        }

        let output_slice =
            unsafe { std::slice::from_raw_parts(output_bstr_ptr.as_ptr(), output_len) };
        let output_string = String::from_utf16_lossy(output_slice);

        trace!("j_morph_result output: {}", output_string);
        Ok(output_string)
    }
}
