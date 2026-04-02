use iced::widget::{button, checkbox, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{field_label, form_shell, helper_text, primary_action, section};
use crate::ui::styles;

#[derive(Debug, Clone, Default)]
pub struct DecodeTabState {
    pub apk_path: String,
    pub output_dir: String,
    pub force: bool,
    pub no_src: bool,
    pub no_res: bool,
}

#[derive(Debug, Clone)]
pub enum DecodeMessage {
    ApkPathChanged(String),
    OutputDirChanged(String),
    BrowseApk,
    BrowseOutputDir,
    ForceToggled(bool),
    NoSrcToggled(bool),
    NoResToggled(bool),
    Start,
}

pub fn view(state: &DecodeTabState, busy: bool) -> Element<'_, DecodeMessage> {
    let content = column![
        text("Decode APK").size(styles::PAGE_TITLE_SIZE),
        helper_text("Open an APK into an apktool project directory for inspection or editing."),
        section(
            "Source",
            Some("Choose the APK to decode and where the extracted project should be written."),
            column![
                field_label("APK file"),
                row![
                    text_input("/path/to/app.apk", &state.apk_path)
                        .on_input(DecodeMessage::ApkPathChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(DecodeMessage::BrowseApk),
                ]
                .spacing(styles::SPACE_8),
                field_label("Output directory"),
                row![
                    text_input("/path/to/output", &state.output_dir)
                        .on_input(DecodeMessage::OutputDirChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(DecodeMessage::BrowseOutputDir),
                ]
                .spacing(styles::SPACE_8),
            ],
        ),
        section(
            "Options",
            Some("Use these only when you need a partial decode or want to overwrite an existing output folder."),
            column![
                checkbox(state.force)
                    .label("Force overwrite existing output")
                    .on_toggle(DecodeMessage::ForceToggled),
                checkbox(state.no_src)
                    .label("Skip sources (-s)")
                    .on_toggle(DecodeMessage::NoSrcToggled),
                checkbox(state.no_res)
                    .label("Skip resources (-r)")
                    .on_toggle(DecodeMessage::NoResToggled),
            ]
            .spacing(styles::SPACE_10),
        ),
        button(text("Decode APK").size(styles::PRIMARY_BUTTON_TEXT_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_10, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(DecodeMessage::Start)),
    ]
    .spacing(styles::SPACE_10);

    form_shell(content).into()
}
