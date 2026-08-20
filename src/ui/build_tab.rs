use iced::widget::{button, checkbox, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{browse_button, field_label, form_shell, primary_action, section};
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
    let mut zipalign_checkbox = checkbox(state.run_zipalign)
        .label("Run zipalign")
        .size(16)
        .text_size(styles::BODY_SIZE);
    if state.can_toggle_zipalign() {
        zipalign_checkbox = zipalign_checkbox.on_toggle(BuildMessage::RunZipalignToggled);
    }

    let mut sign_checkbox = checkbox(state.run_sign)
        .label("Sign APK (apksigner)")
        .size(16)
        .text_size(styles::BODY_SIZE);
    if state.can_toggle_sign() {
        sign_checkbox = sign_checkbox.on_toggle(BuildMessage::RunSignToggled);
    }

    let install_checkbox = checkbox(state.run_install)
        .label("Install via adb (-r)")
        .size(16)
        .text_size(styles::BODY_SIZE)
        .on_toggle(BuildMessage::RunInstallToggled);

    let content = column![
        text("Build APK").size(styles::PAGE_TITLE_SIZE),
        section(
            "Project",
            column![
                field_label("Decoded project directory"),
                row![
                    text_input("/path/to/decoded-project", &state.project_dir)
                        .size(styles::BODY_SIZE)
                        .on_input(BuildMessage::ProjectDirChanged)
                        .width(Length::Fill),
                    browse_button(BuildMessage::BrowseProjectDir),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
                field_label("Output APK (optional)"),
                row![
                    text_input("/path/to/output.apk", &state.output_apk)
                        .size(styles::BODY_SIZE)
                        .on_input(BuildMessage::OutputApkChanged)
                        .width(Length::Fill),
                    browse_button(BuildMessage::BrowseOutputApk),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
            ],
        ),
        section(
            "Build options",
            column![
                checkbox(state.force_all)
                    .label("Force rebuild (-f)")
                    .size(16)
                    .text_size(styles::BODY_SIZE)
                    .on_toggle(BuildMessage::ForceAllToggled),
                checkbox(state.strip_debug_directives)
                    .label("Strip smali debug directives")
                    .size(16)
                    .text_size(styles::BODY_SIZE)
                    .on_toggle(BuildMessage::StripDebugDirectivesToggled),
            ]
            .spacing(styles::SPACE_8),
        ),
        section(
            "Pipeline",
            column![zipalign_checkbox, sign_checkbox, install_checkbox].spacing(styles::SPACE_8),
        ),
        button(text("Build APK").size(styles::BODY_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_6, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(BuildMessage::Start)),
    ]
    .spacing(styles::SECTION_GAP);

    form_shell(content).into()
}
