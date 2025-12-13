use tracing::{debug, info, trace};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::Ime::{
    FELANG_CMODE_HIRAGANAOUT, FELANG_CMODE_NOINVISIBLECHAR, FELANG_CMODE_PRECONV, FELANG_REQ_REV,
};

use crate::felanguage::FElanguage;

use super::converter::Converter;

#[derive(Clone)]
pub struct HiraganaConverter;

impl Converter for HiraganaConverter {
    #[cfg(target_os = "windows")]
    fn convert(&self, text: &str) -> anyhow::Result<String> {
        debug!("Converting to hiragana: {}", text);
        let felanguage = FElanguage::new()?;
        trace!("FElanguage instance created");

        let result = felanguage.j_morph_result(
            text,
            FELANG_REQ_REV,
            FELANG_CMODE_HIRAGANAOUT | FELANG_CMODE_PRECONV | FELANG_CMODE_NOINVISIBLECHAR,
        );

        match &result {
            Ok(converted) => info!("Conversion successful: {} -> {}", text, converted),
            Err(e) => debug!("Conversion failed: {}", e),
        }

        result
    }

    #[cfg(not(target_os = "windows"))]
    fn convert(&self, text: &str) -> anyhow::Result<String> {
        debug!("Converting to hiragana (no-op on non-Windows): {}", text);
        Ok(text.to_string())
    }

    fn name(&self) -> String {
        trace!("Getting converter name");
        "hiragana".to_string()
    }
}
