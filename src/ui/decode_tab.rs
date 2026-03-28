use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length};

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
    let card = column![
        text("Decode APK").size(styles::SECTION_TITLE_SIZE),
        text("Input APK file").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/app.apk", &state.apk_path)
                .on_input(DecodeMessage::ApkPathChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(DecodeMessage::BrowseApk),
        ]
        .spacing(styles::SPACE_8),
        text("Output directory").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/output", &state.output_dir)
                .on_input(DecodeMessage::OutputDirChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(DecodeMessage::BrowseOutputDir),
        ]
        .spacing(styles::SPACE_8),
        text("Options").size(styles::BODY_SIZE),
        column![
            checkbox(state.force)
                .label("Force overwrite")
                .on_toggle(DecodeMessage::ForceToggled),
            checkbox(state.no_src)
                .label("No sources (-s)")
                .on_toggle(DecodeMessage::NoSrcToggled),
            checkbox(state.no_res)
                .label("No resources (-r)")
                .on_toggle(DecodeMessage::NoResToggled),
        ]
        .spacing(styles::SPACE_8),
        button(text("Decode").size(styles::BODY_SIZE + 1.0))
            .width(Length::Fill)
            .padding([styles::SPACE_8, styles::SPACE_16])
            .on_press_maybe((!busy).then_some(DecodeMessage::Start)),
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
