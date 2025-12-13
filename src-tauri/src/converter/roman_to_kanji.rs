use tracing::{debug, info, trace};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::Ime::{
    FELANG_CMODE_HIRAGANAOUT, FELANG_CMODE_NOINVISIBLECHAR, FELANG_CMODE_PRECONV,
    FELANG_CMODE_ROMAN, FELANG_REQ_CONV,
};

use crate::felanguage::FElanguage;

use super::converter::Converter;

pub struct RomanToKanjiConverter;

impl Converter for RomanToKanjiConverter {
    #[cfg(target_os = "windows")]
    fn convert(&self, text: &str) -> anyhow::Result<String> {
        debug!("Converting roman to kanji: {}", text);
        let felanguage = FElanguage::new()?;
        trace!("FElanguage instance created");

        let result = felanguage.j_morph_result(
            text,
            FELANG_REQ_CONV,
            FELANG_CMODE_HIRAGANAOUT
                | FELANG_CMODE_ROMAN
                | FELANG_CMODE_NOINVISIBLECHAR
                | FELANG_CMODE_PRECONV,
        );

        match &result {
            Ok(converted) => info!("Conversion successful: {} -> {}", text, converted),
            Err(e) => debug!("Conversion failed: {}", e),
        }

        result
    }

    #[cfg(not(target_os = "windows"))]
    fn convert(&self, text: &str) -> anyhow::Result<String> {
        debug!("Convert method called on non-Windows platform");
        Err(anyhow::anyhow!(
            "Roman to Kanji conversion is only supported on Windows."
        ))
    }

    fn name(&self) -> String {
        trace!("Getting converter name");
        "roman_to_kanji".to_string()
    }
}
