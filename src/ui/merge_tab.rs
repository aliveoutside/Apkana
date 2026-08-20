use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{browse_button, field_label, form_shell, primary_action, section};
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
    let content = column![
        text("Merge Split APKs").size(styles::PAGE_TITLE_SIZE),
        section(
            "Archive",
            column![
                field_label("Input archive (.apks, .xapk, .apkm)"),
                row![
                    text_input("/path/to/archive.apks", &state.input_path)
                        .size(styles::BODY_SIZE)
                        .on_input(MergeMessage::InputPathChanged)
                        .width(Length::Fill),
                    browse_button(MergeMessage::BrowseInput),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
                field_label("Output APK"),
                row![
                    text_input("/path/to/output.apk", &state.output_path)
                        .size(styles::BODY_SIZE)
                        .on_input(MergeMessage::OutputPathChanged)
                        .width(Length::Fill),
                    browse_button(MergeMessage::BrowseOutput),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
            ],
        ),
        button(text("Merge APK").size(styles::BODY_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_6, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(MergeMessage::Start)),
    ]
    .spacing(styles::SECTION_GAP);

    form_shell(content).into()
}
