use crate::config;

use super::ApkanaApp;

impl ApkanaApp {
    pub(super) fn persist_recent_paths(&mut self) {
        self.config.recent.decode_apk_path = self.decode.apk_path.clone();
        self.config.recent.decode_output_dir = self.decode.output_dir.clone();
        self.config.recent.build_project_dir = self.build.project_dir.clone();
        self.config.recent.build_output_apk = self.build.output_apk.clone();
        self.config.recent.merge_input_path = self.merge.input_path.clone();
        self.config.recent.merge_output_path = self.merge.output_path.clone();
        self.config.recent.install_apk_path = self.install.apk_path.clone();

        if let Err(e) = config::save_config(&self.config) {
            self.push_error(format!("Failed to save recent paths: {e}"));
        }
    }
}
