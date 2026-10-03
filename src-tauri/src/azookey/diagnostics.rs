use std::path::Path;

use anyhow::{Result, ensure};

use super::{azookey_conversion::AzookeyConversion, client::AzookeyConversionClient};

/// Verify native conversion without UI, clipboard access or saved settings.
pub fn check_conversion(resources: &Path, data_dir: &Path) -> Result<()> {
    let client = AzookeyConversionClient::new_with_data_dir(resources, data_dir)?;
    let mut conversion = AzookeyConversion::new(client);
    let converted = conversion.convert("konnichiha")?;
    ensure!(
        !converted.is_empty() && converted != "konnichiha",
        "AzooKey did not convert the probe text"
    );
    println!("AzooKey conversion OK: konnichiha -> {converted}");
    let reconverted = conversion.convert(&converted)?;
    ensure!(
        !reconverted.is_empty(),
        "AzooKey reconversion returned empty text"
    );
    println!("AzooKey reconversion OK: {reconverted}");
    Ok(())
}
