use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{field_label, form_shell, helper_text, primary_action, section};
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
        helper_text("Convert .apks, .xapk, or .apkm archives into a single installable APK."),
        section(
            "Archive",
            Some("Choose the split-package archive and the APK file that Apkana should produce."),
            column![
                field_label("Input archive"),
                row![
                    text_input("/path/to/archive.apks", &state.input_path)
                        .on_input(MergeMessage::InputPathChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(MergeMessage::BrowseInput),
                ]
                .spacing(styles::SPACE_8),
                field_label("Output APK"),
                row![
                    text_input("/path/to/output.apk", &state.output_path)
                        .on_input(MergeMessage::OutputPathChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(MergeMessage::BrowseOutput),
                ]
                .spacing(styles::SPACE_8),
            ],
        ),
        button(text("Merge archive").size(styles::PRIMARY_BUTTON_TEXT_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_10, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(MergeMessage::Start)),
    ]
    .spacing(styles::SPACE_10);

    form_shell(content).into()
}
