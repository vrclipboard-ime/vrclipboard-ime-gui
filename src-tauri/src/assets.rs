use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|asset| asset.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|asset_path| asset_path.starts_with(path))
            .map(|asset_path| SharedString::from(asset_path.into_owned()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::Assets;

    #[test]
    fn embeds_every_lucide_icon_used_by_the_original_ui() {
        let paths = [
            "icons/lucide/github.svg",
            "icons/lucide/external-link.svg",
            "icons/lucide/coffee.svg",
            "icons/lucide/refresh-cw.svg",
            "icons/lucide/check.svg",
            "icons/lucide/alert-circle.svg",
            "icons/lucide/download.svg",
            "icons/lucide/x.svg",
            "icons/lucide/list.svg",
            "icons/lucide/settings.svg",
            "icons/lucide/terminal.svg",
            "icons/lucide/terminal-thin.svg",
            "icons/lucide/bug.svg",
            "icons/lucide/info.svg",
            "icons/lucide/book.svg",
            "icons/lucide/save.svg",
            "icons/lucide/plus.svg",
            "icons/lucide/trash.svg",
            "icons/lucide/chevron-up.svg",
            "icons/lucide/chevron-down.svg",
            "icons/lucide/edit.svg",
            "icons/lucide/align-left.svg",
            "icons/lucide/copy.svg",
            "icons/lucide/minus.svg",
            "icons/lucide/square.svg",
            "icons/lucide/maximize-2.svg",
            "icons/lucide/moon.svg",
            "icons/lucide/sun.svg",
        ];

        for path in paths {
            assert!(
                Assets::get(path).is_some(),
                "missing embedded asset: {path}"
            );
        }
    }
}
