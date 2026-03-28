use iced::Task;

use crate::tools::ToolResult;
use crate::tools::adb::{self, InstallOptions};
use crate::tools::signer::{self, SignOptions};
use crate::tools::zipalign::{self, ZipalignOptions};

use super::{ApkanaApp, Message};

impl ApkanaApp {
    pub(super) fn handle_build_completed(
        &mut self,
        result: Result<ToolResult, String>,
    ) -> Task<Message> {
        self.busy = false;
        self.progress_value = 0.0;
        self.pending_sign_after_build = false;
        self.pending_sign_apk_path = None;
        self.pending_install_after_build = false;
        self.pending_install_after_sign = false;
        self.pending_install_apk_path = None;

        match result {
            Ok(output) if output.exit_code == 0 && self.build.run_zipalign => {
                self.push_result("Build", Ok(output));
                self.push_info("Build succeeded, starting zipalign...");
                self.status_message = String::from("Running zipalign...");

                let input_apk = self.build_output_apk_path();
                let output_apk = input_apk.replace(".apk", "-aligned.apk");

                self.busy = true;
                self.progress_value = 55.0;
                self.pending_sign_after_build = self.build.run_sign;
                self.pending_sign_apk_path = if self.build.run_sign {
                    Some(output_apk.clone())
                } else {
                    None
                };
                self.pending_install_after_build = self.build.run_install;
                self.pending_install_apk_path = if self.build.run_install {
                    Some(output_apk.clone())
                } else {
                    None
                };

                let paths = self.config.tools.clone();
                Task::perform(
                    async move {
                        zipalign::run(
                            &paths,
                            &ZipalignOptions {
                                input_apk,
                                output_apk,
                            },
                        )
                    },
                    Message::ZipalignCompleted,
                )
            }
            Ok(output) if output.exit_code == 0 && self.build.run_sign => {
                self.push_result("Build", Ok(output));
                self.push_info("Build succeeded, starting signing...");
                self.status_message = String::from("Running signing...");
                self.pending_install_after_sign = self.build.run_install;
                self.pending_install_apk_path = if self.build.run_install {
                    Some(self.build_output_apk_path())
                } else {
                    None
                };

                self.start_sign_task(self.build_output_apk_path(), 70.0)
            }
            Ok(output) if output.exit_code == 0 && self.build.run_install => {
                self.push_result("Build", Ok(output));
                self.push_info("Build succeeded, starting adb install...");
                self.status_message = String::from("Running adb install...");
                self.start_install_task(self.build_output_apk_path(), 90.0)
            }
            other => {
                self.push_result("Build", other);
                Task::none()
            }
        }
    }

    pub(super) fn handle_zipalign_completed(
        &mut self,
        result: Result<ToolResult, String>,
    ) -> Task<Message> {
        self.busy = false;
        let mut run_sign = false;
        if let Ok(ref output) = result {
            run_sign = output.exit_code == 0 && self.pending_sign_after_build;
        }
        self.pending_sign_after_build = false;
        self.push_result("Zipalign", result);

        let run_install = self.pending_install_after_build;
        self.pending_install_after_build = false;

        if run_sign {
            let apk_path = self
                .pending_sign_apk_path
                .clone()
                .unwrap_or_else(|| self.sign.apk_path.clone());

            self.push_info("Zipalign succeeded, starting signing...");
            self.status_message = String::from("Running signing...");
            self.pending_sign_apk_path = None;
            self.pending_install_after_sign = run_install;
            self.start_sign_task(apk_path, 85.0)
        } else if run_install {
            let apk_path = self
                .pending_install_apk_path
                .clone()
                .unwrap_or_else(|| self.build_output_apk_path());
            self.pending_install_apk_path = None;
            self.push_info("Zipalign succeeded, starting adb install...");
            self.status_message = String::from("Running adb install...");
            self.start_install_task(apk_path, 90.0)
        } else {
            self.pending_sign_apk_path = None;
            self.pending_install_apk_path = None;
            Task::none()
        }
    }

    pub(super) fn handle_sign_completed(
        &mut self,
        result: Result<ToolResult, String>,
    ) -> Task<Message> {
        self.busy = false;
        self.progress_value = 0.0;
        let run_install = matches!(&result, Ok(output) if output.exit_code == 0)
            && self.pending_install_after_sign;
        self.pending_install_after_sign = false;
        self.push_result("Sign", result);

        if run_install {
            let apk_path = self
                .pending_install_apk_path
                .clone()
                .unwrap_or_else(|| self.build_output_apk_path());
            self.pending_install_apk_path = None;
            self.push_info("Signing succeeded, starting adb install...");
            self.status_message = String::from("Running adb install...");
            self.start_install_task(apk_path, 95.0)
        } else {
            self.pending_install_apk_path = None;
            Task::none()
        }
    }

    pub(super) fn handle_install_completed(
        &mut self,
        result: Result<ToolResult, String>,
    ) -> Task<Message> {
        self.busy = false;
        self.progress_value = 0.0;
        self.push_result("Install", result);
        Task::none()
    }

    pub(super) fn handle_merge_completed(
        &mut self,
        result: Result<ToolResult, String>,
    ) -> Task<Message> {
        self.busy = false;
        self.progress_value = 0.0;
        self.push_result("Merge", result);
        Task::none()
    }

    fn build_output_apk_path(&self) -> String {
        if self.build.output_apk.is_empty() {
            format!("{}/dist/{}.apk", self.build.project_dir, "out")
        } else {
            self.build.output_apk.clone()
        }
    }

    pub(super) fn start_sign_task(
        &mut self,
        apk_path: String,
        progress_value: f32,
    ) -> Task<Message> {
        if let Err(error) = self.validate_sign_inputs(&apk_path) {
            self.status_message = error.clone();
            self.push_error(error);
            return Task::none();
        }

        self.busy = true;
        self.progress_value = progress_value;

        let options = if self.use_default_debug_signing() {
            let keystore_path = match Self::default_debug_keystore_path() {
                Some(path) => path.to_string_lossy().into_owned(),
                None => {
                    self.busy = false;
                    self.progress_value = 0.0;
                    self.status_message =
                        String::from("Sign: default debug keystore path is unavailable");
                    self.push_error("Sign: default debug keystore path is unavailable");
                    return Task::none();
                }
            };

            self.push_info(format!(
                "No keystore selected; using default debug key at {keystore_path}"
            ));

            SignOptions {
                apk_path,
                keystore_path,
                alias: String::from("androiddebugkey"),
                keystore_pass: String::from("android"),
                key_pass: String::from("android"),
            }
        } else {
            SignOptions {
                apk_path,
                keystore_path: self.sign.keystore_path.clone(),
                alias: self.sign.alias.clone(),
                keystore_pass: self.sign.keystore_pass.clone(),
                key_pass: self.sign.key_pass.clone(),
            }
        };

        let paths = self.config.tools.clone();
        Task::perform(
            async move { signer::sign(&paths, &options) },
            Message::SignCompleted,
        )
    }

    pub(super) fn start_install_task(
        &mut self,
        apk_path: String,
        progress_value: f32,
    ) -> Task<Message> {
        self.busy = true;
        self.progress_value = progress_value;

        let options = InstallOptions {
            apk_path,
            replace_existing: true,
        };
        let paths = self.config.tools.clone();
        Task::perform(
            async move { adb::install(&paths, &options) },
            Message::InstallCompleted,
        )
    }
}
