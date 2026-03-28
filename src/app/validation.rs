use std::path::PathBuf;

use super::ApkanaApp;

impl ApkanaApp {
    pub(super) fn validate_decode_inputs(&self) -> Result<(), String> {
        if self.decode.apk_path.trim().is_empty() {
            return Err(String::from("Decode: APK path is required"));
        }
        if self.decode.output_dir.trim().is_empty() {
            return Err(String::from("Decode: output directory is required"));
        }
        Ok(())
    }

    pub(super) fn validate_build_inputs(&self) -> Result<(), String> {
        if self.build.project_dir.trim().is_empty() {
            return Err(String::from("Build: decoded project directory is required"));
        }
        if self.build.run_sign {
            let apk_path = if self.build.output_apk.trim().is_empty() {
                format!("{}/dist/{}.apk", self.build.project_dir, "out")
            } else {
                self.build.output_apk.clone()
            };
            self.validate_sign_inputs(&apk_path)?;
        }
        Ok(())
    }

    pub(super) fn validate_sign_inputs(&self, apk_path: &str) -> Result<(), String> {
        if apk_path.trim().is_empty() {
            return Err(String::from("Sign: APK path is required"));
        }

        if self.use_default_debug_signing() {
            let Some(path) = Self::default_debug_keystore_path() else {
                return Err(String::from(
                    "Sign: no keystore selected and home directory is unavailable",
                ));
            };

            if !path.is_file() {
                return Err(format!(
                    "Sign: no keystore selected. Default debug keystore was not found at {}",
                    path.display()
                ));
            }

            return Ok(());
        }

        if self.sign.keystore_path.trim().is_empty() {
            return Err(String::from("Sign: keystore path is required"));
        }
        if self.sign.alias.trim().is_empty() {
            return Err(String::from("Sign: key alias is required"));
        }
        Ok(())
    }

    pub(super) fn validate_merge_inputs(&self) -> Result<(), String> {
        if self.merge.input_path.trim().is_empty() {
            return Err(String::from("Merge: input archive path is required"));
        }
        if self.merge.output_path.trim().is_empty() {
            return Err(String::from("Merge: output APK path is required"));
        }

        let input = self.merge.input_path.trim().to_ascii_lowercase();
        if !(input.ends_with(".apks") || input.ends_with(".xapk") || input.ends_with(".apkm")) {
            return Err(String::from(
                "Merge: input file must have .apks, .xapk, or .apkm extension",
            ));
        }

        Ok(())
    }

    pub(super) fn validate_install_inputs(&self) -> Result<(), String> {
        if self.install.apk_path.trim().is_empty() {
            return Err(String::from("Install: APK path is required"));
        }
        Ok(())
    }

    pub(super) fn use_default_debug_signing(&self) -> bool {
        self.sign.keystore_path.trim().is_empty()
            && self.sign.alias.trim().is_empty()
            && self.sign.keystore_pass.trim().is_empty()
            && self.sign.key_pass.trim().is_empty()
    }

    pub(super) fn default_debug_keystore_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".android").join("debug.keystore"))
    }
}
