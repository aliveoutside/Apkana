use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{field_label, form_shell, helper_text, primary_action, section};
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
        helper_text("Use your own keystore or fall back to the standard Android debug keystore."),
        section(
            "Target",
            Some("Choose the APK that should be signed."),
            column![
                field_label("APK file"),
                row![
                    text_input("/path/to/app.apk", &state.apk_path)
                        .on_input(SignMessage::ApkPathChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(SignMessage::BrowseApk),
                ]
                .spacing(styles::SPACE_8),
            ],
        ),
        section(
            "Signing credentials",
            Some("Leave every signing field empty to use ~/.android/debug.keystore."),
            column![
                field_label("Keystore (optional)"),
                row![
                    text_input("/path/to/keystore.jks", &state.keystore_path)
                        .on_input(SignMessage::KeystorePathChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(SignMessage::BrowseKeystore),
                ]
                .spacing(styles::SPACE_8),
                field_label("Alias (optional)"),
                text_input("Key alias", &state.alias)
                    .on_input(SignMessage::AliasChanged)
                    .width(Length::Fill),
                field_label("Keystore password (optional)"),
                text_input("Keystore password", &state.keystore_pass)
                    .secure(true)
                    .on_input(SignMessage::KeystorePassChanged)
                    .width(Length::Fill),
                field_label("Key password (optional)"),
                text_input("Key password", &state.key_pass)
                    .secure(true)
                    .on_input(SignMessage::KeyPassChanged)
                    .width(Length::Fill),
            ]
            .spacing(styles::SPACE_8),
        ),
        button(text("Sign APK").size(styles::PRIMARY_BUTTON_TEXT_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_10, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(SignMessage::Start)),
    ]
    .spacing(styles::SPACE_10);

    form_shell(content).into()
}
