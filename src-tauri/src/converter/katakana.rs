use tracing::{debug, info, trace};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::Ime::{
    FELANG_CMODE_KATAKANAOUT, FELANG_CMODE_NOINVISIBLECHAR, FELANG_CMODE_PRECONV, FELANG_REQ_REV,
};

use crate::felanguage::FElanguage;

use super::converter::Converter;

#[derive(Clone)]
pub struct KatakanaConverter;

impl Converter for KatakanaConverter {
    #[cfg(target_os = "windows")]
    fn convert(&self, text: &str) -> anyhow::Result<String> {
        debug!("Converting to katakana: {}", text);
        let felanguage = FElanguage::new()?;
        trace!("FElanguage instance created");

        let result = felanguage.j_morph_result(
            text,
            FELANG_REQ_REV,
            FELANG_CMODE_KATAKANAOUT | FELANG_CMODE_PRECONV | FELANG_CMODE_NOINVISIBLECHAR,
        );

        match &result {
            Ok(converted) => info!("Conversion successful: {} -> {}", text, converted),
            Err(e) => debug!("Conversion failed: {}", e),
        }

        result
    }

    #[cfg(not(target_os = "windows"))]
    fn convert(&self, text: &str) -> anyhow::Result<String> {
        debug!("Converting to katakana (no-op on non-Windows): {}", text);
        Ok(text.to_string())
    }

    fn name(&self) -> String {
        trace!("Getting converter name");
        "katakana".to_string()
    }
}
