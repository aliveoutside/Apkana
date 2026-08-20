use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{browse_button, field_label, form_shell, primary_action, section};
use crate::ui::styles;

#[derive(Debug, Clone, Default)]
pub struct SignTabState {
    pub apk_path: String,
    pub keystore_path: String,
    pub alias: String,
    pub keystore_pass: String,
    pub key_pass: String,
}

#[derive(Debug, Clone)]
pub enum SignMessage {
    ApkPathChanged(String),
    KeystorePathChanged(String),
    AliasChanged(String),
    KeystorePassChanged(String),
    KeyPassChanged(String),
    BrowseApk,
    BrowseKeystore,
    Start,
}

pub fn view(state: &SignTabState, busy: bool) -> Element<'_, SignMessage> {
    let content = column![
        text("Sign APK").size(styles::PAGE_TITLE_SIZE),
        section(
            "Target APK",
            column![
                field_label("APK file"),
                row![
                    text_input("/path/to/app.apk", &state.apk_path)
                        .size(styles::BODY_SIZE)
                        .on_input(SignMessage::ApkPathChanged)
                        .width(Length::Fill),
                    browse_button(SignMessage::BrowseApk),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
            ],
        ),
        section(
            "Keystore",
            column![
                field_label("Keystore file (leave empty for debug.keystore)"),
                row![
                    text_input("/path/to/keystore.jks", &state.keystore_path)
                        .size(styles::BODY_SIZE)
                        .on_input(SignMessage::KeystorePathChanged)
                        .width(Length::Fill),
                    browse_button(SignMessage::BrowseKeystore),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
                field_label("Key alias"),
                text_input("Key alias (optional)", &state.alias)
                    .size(styles::BODY_SIZE)
                    .on_input(SignMessage::AliasChanged)
                    .width(Length::Fill),
                field_label("Keystore password"),
                text_input("Keystore password (optional)", &state.keystore_pass)
                    .size(styles::BODY_SIZE)
                    .secure(true)
                    .on_input(SignMessage::KeystorePassChanged)
                    .width(Length::Fill),
                field_label("Key password"),
                text_input("Key password (optional)", &state.key_pass)
                    .size(styles::BODY_SIZE)
                    .secure(true)
                    .on_input(SignMessage::KeyPassChanged)
                    .width(Length::Fill),
            ]
            .spacing(styles::SPACE_6),
        ),
        button(text("Sign APK").size(styles::BODY_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_6, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(SignMessage::Start)),
    ]
    .spacing(styles::SECTION_GAP);

    form_shell(content).into()
}
