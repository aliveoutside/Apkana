use iced::Task;
use std::path::Path;

use crate::config;
use crate::tools::apktool::{self, BuildOptions, DecodeOptions};
use crate::tools::merger::{self, MergeOptions};
use crate::ui::build_tab::BuildMessage;
use crate::ui::decode_tab::DecodeMessage;
use crate::ui::install_tab::InstallMessage;
use crate::ui::merge_tab::MergeMessage;
use crate::ui::settings_modal::SettingsMessage;
use crate::ui::sign_tab::SignMessage;
use crate::utils;

use super::{ApkanaApp, Message};

impl ApkanaApp {
    pub(super) fn update_decode(&mut self, message: DecodeMessage) -> Task<Message> {
        match message {
            DecodeMessage::ApkPathChanged(v) => self.decode.apk_path = v,
            DecodeMessage::OutputDirChanged(v) => self.decode.output_dir = v,
            DecodeMessage::BrowseApk => {
                if let Some(path) = utils::pick_file() {
                    self.decode.apk_path = path;
                    self.persist_recent_paths();
                }
            }
            DecodeMessage::BrowseOutputDir => {
                if let Some(path) = utils::pick_folder() {
                    self.decode.output_dir = path;
                    self.persist_recent_paths();
                }
            }
            DecodeMessage::ForceToggled(v) => self.decode.force = v,
            DecodeMessage::NoSrcToggled(v) => self.decode.no_src = v,
            DecodeMessage::NoResToggled(v) => self.decode.no_res = v,
            DecodeMessage::Start => {
                if let Err(error) = self.validate_decode_inputs() {
                    self.status_message = error.clone();
                    self.push_error(error);
                    return Task::none();
                }
                self.persist_recent_paths();
                self.busy = true;
                self.progress_value = 20.0;
                self.status_message = String::from("Running decode...");
                self.push_info("Starting decode...");
                let options = DecodeOptions {
                    apk_path: self.decode.apk_path.clone(),
                    output_dir: self.decode.output_dir.clone(),
                    force: self.decode.force,
                    no_src: self.decode.no_src,
                    no_res: self.decode.no_res,
                };
                let paths = self.config.tools.clone();
                return Task::perform(
                    async move { apktool::decode(&paths, &options) },
                    Message::DecodeCompleted,
                );
            }
        }

        Task::none()
    }

    pub(super) fn update_build(&mut self, message: BuildMessage) -> Task<Message> {
        match message {
            BuildMessage::ProjectDirChanged(v) => self.build.project_dir = v,
            BuildMessage::OutputApkChanged(v) => self.build.output_apk = v,
            BuildMessage::BrowseProjectDir => {
                if let Some(path) = utils::pick_folder() {
                    self.build.project_dir = path;
                    self.persist_recent_paths();
                }
            }
            BuildMessage::BrowseOutputApk => {
                if let Some(path) = utils::save_file() {
                    self.build.output_apk = path;
                    self.persist_recent_paths();
                }
            }
            BuildMessage::ForceAllToggled(v) => self.build.force_all = v,
            BuildMessage::StripDebugDirectivesToggled(v) => self.build.strip_debug_directives = v,
            BuildMessage::RunZipalignToggled(v) => {
                self.build.run_zipalign = v;
                self.normalize_build_pipeline_toggles();
            }
            BuildMessage::RunSignToggled(v) => {
                self.build.run_sign = v;
                self.normalize_build_pipeline_toggles();
            }
            BuildMessage::RunInstallToggled(v) => {
                self.build.run_install = v;
                self.normalize_build_pipeline_toggles();
            }
            BuildMessage::Start => {
                if let Err(error) = self.validate_build_inputs() {
                    self.status_message = error.clone();
                    self.push_error(error);
                    return Task::none();
                }
                self.persist_recent_paths();
                self.busy = true;
                self.progress_value = 25.0;
                self.status_message = String::from("Running build...");
                self.pending_sign_after_build = false;
                self.pending_sign_apk_path = None;
                self.pending_install_after_build = false;
                self.pending_install_after_sign = false;
                self.pending_install_apk_path = None;
                self.push_info("Starting build...");
                let options = BuildOptions {
                    project_dir: self.build.project_dir.clone(),
                    output_apk: self.build.output_apk.clone(),
                    force_all: self.build.force_all,
                    strip_debug_directives: self.build.strip_debug_directives,
                };
                let paths = self.config.tools.clone();
                return Task::perform(
                    async move { apktool::build(&paths, &options) },
                    Message::BuildCompleted,
                );
            }
        }
        Task::none()
    }

    fn normalize_build_pipeline_toggles(&mut self) {
        if self.build.run_install {
            self.build.run_sign = true;
            self.build.run_zipalign = true;
            return;
        }

        if self.build.run_sign {
            self.build.run_zipalign = true;
        }
    }

    pub(super) fn update_sign(&mut self, message: SignMessage) -> Task<Message> {
        match message {
            SignMessage::ApkPathChanged(v) => self.sign.apk_path = v,
            SignMessage::KeystorePathChanged(v) => self.sign.keystore_path = v,
            SignMessage::AliasChanged(v) => self.sign.alias = v,
            SignMessage::KeystorePassChanged(v) => self.sign.keystore_pass = v,
            SignMessage::KeyPassChanged(v) => self.sign.key_pass = v,
            SignMessage::BrowseApk => {
                if let Some(path) = utils::pick_file() {
                    self.sign.apk_path = path;
                }
            }
            SignMessage::BrowseKeystore => {
                if let Some(path) = utils::pick_file() {
                    self.sign.keystore_path = path;
                }
            }
            SignMessage::Start => {
                self.status_message = String::from("Running signing...");
                self.push_info("Starting signing...");
                return self.start_sign_task(self.sign.apk_path.clone(), 40.0);
            }
        }
        Task::none()
    }

    pub(super) fn update_merge(&mut self, message: MergeMessage) -> Task<Message> {
        match message {
            MergeMessage::InputPathChanged(v) => self.merge.input_path = v,
            MergeMessage::OutputPathChanged(v) => self.merge.output_path = v,
            MergeMessage::BrowseInput => {
                if let Some(path) = utils::pick_split_archive_file() {
                    self.merge.input_path = path;
                    if let Some(suggested) = suggested_merge_output_path(&self.merge.input_path) {
                        self.merge.output_path = suggested;
                    }
                    self.persist_recent_paths();
                }
            }
            MergeMessage::BrowseOutput => {
                if let Some(path) = utils::save_apk_file_with_default(&self.merge.output_path) {
                    self.merge.output_path = path;
                    self.persist_recent_paths();
                }
            }
            MergeMessage::Start => {
                if let Err(error) = self.validate_merge_inputs() {
                    self.status_message = error.clone();
                    self.push_error(error);
                    return Task::none();
                }

                self.persist_recent_paths();
                self.busy = true;
                self.progress_value = 30.0;
                self.status_message = String::from("Running merge...");
                self.push_info("Starting merge...");

                let options = MergeOptions {
                    input_path: self.merge.input_path.clone(),
                    output_path: self.merge.output_path.clone(),
                };

                return Task::perform(
                    async move { merger::merge(options) },
                    Message::MergeCompleted,
                );
            }
        }

        Task::none()
    }

    pub(super) fn update_install(&mut self, message: InstallMessage) -> Task<Message> {
        match message {
            InstallMessage::ApkPathChanged(v) => self.install.apk_path = v,
            InstallMessage::BrowseApk => {
                if let Some(path) = utils::pick_apk_file() {
                    self.install.apk_path = path;
                    self.persist_recent_paths();
                }
            }
            InstallMessage::Start => {
                if let Err(error) = self.validate_install_inputs() {
                    self.status_message = error.clone();
                    self.push_error(error);
                    return Task::none();
                }

                self.persist_recent_paths();
                self.status_message = String::from("Running adb install...");
                self.push_info("Starting adb install...");
                return self.start_install_task(self.install.apk_path.clone(), 40.0);
            }
        }

        Task::none()
    }

    pub(super) fn update_settings(&mut self, message: SettingsMessage) -> Task<Message> {
        match message {
            SettingsMessage::JavaChanged(v) => {
                self.settings_draft
                    .get_or_insert_with(|| self.config.tools.clone())
                    .java = v;
            }
            SettingsMessage::ApktoolJarChanged(v) => {
                self.settings_draft
                    .get_or_insert_with(|| self.config.tools.clone())
                    .apktool_jar = v;
            }
            SettingsMessage::ApksignerChanged(v) => {
                self.settings_draft
                    .get_or_insert_with(|| self.config.tools.clone())
                    .apksigner = v;
            }
            SettingsMessage::ZipalignChanged(v) => {
                self.settings_draft
                    .get_or_insert_with(|| self.config.tools.clone())
                    .zipalign = v;
            }
            SettingsMessage::AdbChanged(v) => {
                self.settings_draft
                    .get_or_insert_with(|| self.config.tools.clone())
                    .adb = v;
            }
            SettingsMessage::BrowseApktoolJar => {
                if let Some(path) = utils::pick_file() {
                    self.settings_draft
                        .get_or_insert_with(|| self.config.tools.clone())
                        .apktool_jar = path;
                }
            }
            SettingsMessage::Save => {
                if let Some(new_tools) = self.settings_draft.take() {
                    self.config.tools = new_tools;
                }
                match config::save_config(&self.config) {
                    Ok(_) => {
                        self.status_message = String::from("Settings saved");
                        self.push_info("Settings saved");
                    }
                    Err(e) => self.push_error(format!("Settings save failed: {e}")),
                }
                self.show_settings = false;
            }
            SettingsMessage::Cancel => {
                self.settings_draft = None;
                self.show_settings = false;
            }
        }
        Task::none()
    }
}

fn suggested_merge_output_path(input_path: &str) -> Option<String> {
    let path = Path::new(input_path);
    let parent = path.parent()?;
    let stem = path.file_stem()?.to_string_lossy();
    Some(
        parent
            .join(format!("{stem}_merged.apk"))
            .to_string_lossy()
            .into_owned(),
    )
}
