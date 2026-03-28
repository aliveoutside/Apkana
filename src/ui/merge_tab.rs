use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::ui::styles;

#[derive(Debug, Clone, Default)]
pub struct MergeTabState {
    pub input_path: String,
    pub output_path: String,
}

#[derive(Debug, Clone)]
pub enum MergeMessage {
    InputPathChanged(String),
    OutputPathChanged(String),
    BrowseInput,
    BrowseOutput,
    Start,
}

pub fn view(state: &MergeTabState, busy: bool) -> Element<'_, MergeMessage> {
    let card = column![
        text("Merge Split APKs").size(styles::SECTION_TITLE_SIZE),
        text("Converts .apks/.xapk/.apkm archives into a single APK")
            .size(styles::BODY_SIZE)
            .style(text::secondary),
        text("Input archive").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/archive.apks", &state.input_path)
                .on_input(MergeMessage::InputPathChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(MergeMessage::BrowseInput),
        ]
        .spacing(styles::SPACE_8),
        text("Output APK").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/output.apk", &state.output_path)
                .on_input(MergeMessage::OutputPathChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(MergeMessage::BrowseOutput),
        ]
        .spacing(styles::SPACE_8),
        button(text("Merge").size(styles::BODY_SIZE + 1.0))
            .width(Length::Fill)
            .padding([styles::SPACE_8, styles::SPACE_16])
            .on_press_maybe((!busy).then_some(MergeMessage::Start)),
    ]
    .spacing(styles::SPACE_8);

    container(
        container(card)
            .style(container::rounded_box)
            .padding(styles::SPACE_16),
    )
    .max_width(styles::MAX_FORM_WIDTH)
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}
