//! Android SDK integration checks. Run: cargo test --test merge_archives -- --ignored
use apkana::config::ToolPaths;
use apkana::tools::merger::{self, MergeOptions};
use apkana::tools::signer::SignOptions;
use std::path::Path;
use std::process::Command;

fn signing(dir: &Path) -> SignOptions {
    let key = dir.join("test.jks");
    assert!(
        Command::new("keytool")
            .args(["-genkeypair", "-keystore"])
            .arg(&key)
            .args([
                "-storepass",
                "android",
                "-keypass",
                "android",
                "-alias",
                "test",
                "-keyalg",
                "RSA",
                "-validity",
                "10000",
                "-dname",
                "CN=Apkana Test"
            ])
            .output()
            .unwrap()
            .status
            .success()
    );
    SignOptions {
        apk_path: String::new(),
        keystore_path: key.to_string_lossy().into_owned(),
        alias: "test".into(),
        keystore_pass: "android".into(),
        key_pass: "android".into(),
    }
}

#[test]
#[ignore = "requires Android SDK build tools and Java on PATH"]
fn real_archives_preserve_feature_code_manifest_resources_and_assets() {
    let dir = tempfile::tempdir().unwrap();
    let key = signing(dir.path());
    for ext in ["apks", "xapk", "apkm"] {
        let out = dir.path().join(format!("merged-{ext}.apk"));
        merger::merge(
            &ToolPaths::default(),
            MergeOptions {
                input_path: format!(
                    "{}/tests/fixtures/feature.{ext}",
                    env!("CARGO_MANIFEST_DIR")
                ),
                output_path: out.to_string_lossy().into_owned(),
            },
            key.clone(),
        )
        .unwrap();
        let manifest = Command::new("aapt2")
            .args(["dump", "xmltree"])
            .arg(&out)
            .args(["--file", "AndroidManifest.xml"])
            .output()
            .unwrap();
        assert!(manifest.status.success());
        let xml = String::from_utf8_lossy(&manifest.stdout);
        for name in [
            "FeatureActivity",
            "FeatureService",
            "FeatureReceiver",
            "FeatureProvider",
            "android.permission.INTERNET",
            "extractNativeLibs",
        ] {
            assert!(xml.contains(name), "missing {name}: {xml}");
        }
        for removed in [
            "isFeatureSplit",
            "isSplitRequired",
            "requiredSplitTypes",
            "splitTypes",
        ] {
            assert!(
                !xml.contains(removed),
                "split dependency remains: {removed}"
            );
        }
        let dump = Command::new("aapt2")
            .args(["dump", "resources"])
            .arg(&out)
            .output()
            .unwrap();
        let resources = String::from_utf8_lossy(&dump.stdout);
        for name in ["bonjour-feature", "feature-resource-ok", "hdpi", "xhdpi"] {
            assert!(
                resources.contains(name),
                "missing resource {name}: {resources}"
            );
        }
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&out).unwrap()).unwrap();
        for name in [
            "classes.dex",
            "classes2.dex",
            "assets/base.txt",
            "assets/feature.txt",
            "lib/x86_64/libprobe.so",
        ] {
            assert!(archive.by_name(name).is_ok(), "missing {name}");
        }
        if let Ok(destination) = std::env::var("APKANA_ANDROID_TEST_OUTPUT") {
            std::fs::copy(&out, destination).unwrap();
        }
    }
    // A late validation failure must not publish or replace an existing APK.
    let out = dir.path().join("previous.apk");
    std::fs::write(&out, b"previous-good-output").unwrap();
    let paths = ToolPaths {
        aapt2: dir
            .path()
            .join("missing-aapt2")
            .to_string_lossy()
            .into_owned(),
        ..ToolPaths::default()
    };
    let result = merger::merge(
        &paths,
        MergeOptions {
            input_path: format!("{}/tests/fixtures/feature.apks", env!("CARGO_MANIFEST_DIR")),
            output_path: out.to_string_lossy().into_owned(),
        },
        key,
    );
    assert!(result.unwrap_err().contains("aapt2"));
    assert_eq!(std::fs::read(out).unwrap(), b"previous-good-output");
}

#[test]
fn ambiguous_variants_do_not_replace_existing_output() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.apk");
    std::fs::write(&out, b"previous-good-output").unwrap();
    let result = merger::merge(
        &ToolPaths::default(),
        MergeOptions {
            input_path: format!(
                "{}/tests/fixtures/variants.apks",
                env!("CARGO_MANIFEST_DIR")
            ),
            output_path: out.to_string_lossy().into_owned(),
        },
        SignOptions {
            apk_path: String::new(),
            keystore_path: String::new(),
            alias: String::new(),
            keystore_pass: String::new(),
            key_pass: String::new(),
        },
    );
    assert!(result.unwrap_err().contains("variants"));
    assert_eq!(std::fs::read(out).unwrap(), b"previous-good-output");
}
