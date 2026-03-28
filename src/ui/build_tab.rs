use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length};

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

    let mut sign_checkbox = checkbox(state.run_sign).label("Run sign after build");
    if state.can_toggle_sign() {
        sign_checkbox = sign_checkbox.on_toggle(BuildMessage::RunSignToggled);
    }

    let install_checkbox = checkbox(state.run_install)
        .label("Auto-install via adb after pipeline")
        .on_toggle(BuildMessage::RunInstallToggled);

    let card = column![
        text("Build APK").size(styles::SECTION_TITLE_SIZE),
        text("Decoded project directory").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/decoded-project", &state.project_dir)
                .on_input(BuildMessage::ProjectDirChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(BuildMessage::BrowseProjectDir),
        ]
        .spacing(styles::SPACE_8),
        text("Output APK (optional)").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/output.apk", &state.output_apk)
                .on_input(BuildMessage::OutputApkChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(BuildMessage::BrowseOutputApk),
        ]
        .spacing(styles::SPACE_8),
        text("Build options").size(styles::BODY_SIZE),
        checkbox(state.force_all)
            .label("Force rebuild (-f)")
            .on_toggle(BuildMessage::ForceAllToggled),
        checkbox(state.strip_debug_directives)
            .label("OOM workaround: strip smali debug directives")
            .on_toggle(BuildMessage::StripDebugDirectivesToggled),
        text("Pipeline").size(styles::BODY_SIZE),
        column![zipalign_checkbox, sign_checkbox, install_checkbox,].spacing(styles::SPACE_8),
        button(text("Build").size(styles::BODY_SIZE + 1.0))
            .width(Length::Fill)
            .padding([styles::SPACE_8, styles::SPACE_16])
            .on_press_maybe((!busy).then_some(BuildMessage::Start))
    ]
    .spacing(styles::SPACE_8);

    container(
        container(card)
            .style(container::rounded_box)
            .padding(styles::SPACE_16),
    )
    .width(Length::Fill)
    .into()
}
