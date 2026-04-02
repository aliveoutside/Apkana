use iced::widget::{button, checkbox, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{field_label, form_shell, helper_text, primary_action, section};
use crate::ui::styles;

#[derive(Debug, Clone, Default)]
pub struct BuildTabState {
    pub project_dir: String,
    pub output_apk: String,
    pub force_all: bool,
    pub strip_debug_directives: bool,
    pub run_zipalign: bool,
    pub run_sign: bool,
    pub run_install: bool,
}

impl BuildTabState {
    pub fn can_toggle_zipalign(&self) -> bool {
        !self.run_sign && !self.run_install
    }

    pub fn can_toggle_sign(&self) -> bool {
        !self.run_install
    }
}

#[derive(Debug, Clone)]
pub enum BuildMessage {
    ProjectDirChanged(String),
    OutputApkChanged(String),
    BrowseProjectDir,
    BrowseOutputApk,
    ForceAllToggled(bool),
    StripDebugDirectivesToggled(bool),
    RunZipalignToggled(bool),
    RunSignToggled(bool),
    RunInstallToggled(bool),
    Start,
}

pub fn view(state: &BuildTabState, busy: bool) -> Element<'_, BuildMessage> {
    let mut zipalign_checkbox = checkbox(state.run_zipalign).label("Run zipalign after build");
    if state.can_toggle_zipalign() {
        zipalign_checkbox = zipalign_checkbox.on_toggle(BuildMessage::RunZipalignToggled);
    }

    let mut sign_checkbox = checkbox(state.run_sign).label("Run signing after zipalign");
    if state.can_toggle_sign() {
        sign_checkbox = sign_checkbox.on_toggle(BuildMessage::RunSignToggled);
    }

    let install_checkbox = checkbox(state.run_install)
        .label("Auto-install with adb after the pipeline finishes")
        .on_toggle(BuildMessage::RunInstallToggled);

    let content = column![
        text("Build APK").size(styles::PAGE_TITLE_SIZE),
        helper_text("Turn a decoded apktool project back into an APK, with optional post-build steps."),
        section(
            "Project",
            Some("Select the decoded project directory. You can optionally choose an explicit output APK path."),
            column![
                field_label("Decoded project directory"),
                row![
                    text_input("/path/to/decoded-project", &state.project_dir)
                        .on_input(BuildMessage::ProjectDirChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(BuildMessage::BrowseProjectDir),
                ]
                .spacing(styles::SPACE_8),
                field_label("Output APK (optional)"),
                row![
                    text_input("/path/to/output.apk", &state.output_apk)
                        .on_input(BuildMessage::OutputApkChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(BuildMessage::BrowseOutputApk),
                ]
                .spacing(styles::SPACE_8),
            ],
        ),
        section(
            "Build options",
            Some("These affect how apktool rebuilds the package before any signing or install steps run."),
            column![
                checkbox(state.force_all)
                    .label("Force rebuild (-f)")
                    .on_toggle(BuildMessage::ForceAllToggled),
                checkbox(state.strip_debug_directives)
                    .label("Strip smali debug directives to reduce OOM risk")
                    .on_toggle(BuildMessage::StripDebugDirectivesToggled),
            ]
            .spacing(styles::SPACE_10),
        ),
        section(
            "Pipeline",
            Some("Later steps depend on earlier ones. Install implies sign, and sign implies zipalign."),
            column![zipalign_checkbox, sign_checkbox, install_checkbox].spacing(styles::SPACE_10),
        ),
        button(text("Build APK").size(styles::PRIMARY_BUTTON_TEXT_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_10, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(BuildMessage::Start)),
    ]
    .spacing(styles::SPACE_10);

    form_shell(content).into()
}
