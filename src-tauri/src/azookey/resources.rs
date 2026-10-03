use std::{fs, path::Path};

use anyhow::{Context, Result, ensure};

// SwiftPM's Bundle.module searches beside the executable, not beside the DLL.
// Never allow its compiled-in developer build path to mask missing resources.
const RESOURCE_BUNDLES: &[(&str, &[&str])] = &[
    (
        "AzooKeyKanaKanjiConverter_KanaKanjiConverterModuleWithDefaultDictionary.resources",
        &[
            "Dictionary/mm.binary",
            "EmojiDictionary/emoji_all_E15.1.txt",
        ],
    ),
    (
        "AzooKeyKanaKanjiConverter_EfficientNGram.resources",
        &["tokenizer/tokenizer.json", "tokenizer/vocab.json"],
    ),
    (
        "swift-transformers_Hub.resources",
        &["gpt2_tokenizer_config.json", "t5_tokenizer_config.json"],
    ),
];

pub(super) fn prepare_resource_bundles(native_dir: &Path, executable_dir: &Path) -> Result<()> {
    for &(name, markers) in RESOURCE_BUNDLES {
        let destination = executable_dir.join(name);
        if validate_bundle(&destination, markers).is_ok() {
            continue;
        }
        // Also support the old layout and cargo run without requiring users to
        // install Swift or create directories from the developer's computer.
        let source = native_dir.join(name);
        validate_bundle(&source, markers).with_context(|| {
            format!(
                "AzooKey resource bundle missing or incomplete: {}",
                source.display()
            )
        })?;
        copy_directory(&source, &destination).with_context(|| {
            format!("Cannot place AzooKey resources beside the executable: {}. Extract the complete distribution into a writable folder.", destination.display())
        })?;
        validate_bundle(&destination, markers)?;
    }
    Ok(())
}

fn validate_bundle(directory: &Path, markers: &[&str]) -> Result<()> {
    for marker in markers {
        let file = directory.join(marker);
        let metadata = fs::metadata(&file)
            .with_context(|| format!("Missing AzooKey resource: {}", file.display()))?;
        ensure!(
            metadata.is_file() && metadata.len() > 0,
            "Invalid AzooKey resource: {}",
            file.display()
        );
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let destination = destination.join(entry.file_name());
        if kind.is_dir() {
            copy_directory(&entry.path(), &destination)?;
        } else {
            ensure!(
                kind.is_file(),
                "Unexpected resource entry: {}",
                entry.path().display()
            );
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new() -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "vrclipboard-resources-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }

        fn populate(&self, directory: &Path) {
            for &(name, markers) in RESOURCE_BUNDLES {
                for marker in markers {
                    let file = directory.join(name).join(marker);
                    fs::create_dir_all(file.parent().unwrap()).unwrap();
                    fs::write(file, b"test resource").unwrap();
                }
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn old_layout_is_repaired_beside_the_executable_in_a_unicode_path() {
        let fixture = Fixture::new();
        let executable_dir = fixture.0.join("別の PC 配布フォルダー");
        let native_dir = executable_dir.join("azookey-native");
        fixture.populate(&native_dir);
        prepare_resource_bundles(&native_dir, &executable_dir).unwrap();
        for &(name, markers) in RESOURCE_BUNDLES {
            validate_bundle(&executable_dir.join(name), markers).unwrap();
        }
    }

    #[test]
    fn packaged_resources_do_not_require_a_developer_build_directory() {
        let fixture = Fixture::new();
        fixture.populate(&fixture.0);
        prepare_resource_bundles(&fixture.0.join("nonexistent-native-bundles"), &fixture.0)
            .unwrap();
    }

    #[test]
    fn incomplete_distribution_is_rejected_before_entering_swift() {
        let fixture = Fixture::new();
        let native_dir = fixture.0.join("azookey-native");
        fixture.populate(&native_dir);
        let marker = native_dir
            .join(RESOURCE_BUNDLES[0].0)
            .join(RESOURCE_BUNDLES[0].1[0]);
        fs::write(marker, b"").unwrap();
        assert!(prepare_resource_bundles(&native_dir, &fixture.0).is_err());
    }
}
