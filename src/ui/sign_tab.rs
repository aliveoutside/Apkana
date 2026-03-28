use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

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
    let card = column![
        text("Sign APK").size(styles::SECTION_TITLE_SIZE),
        text("APK file").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/app.apk", &state.apk_path)
                .on_input(SignMessage::ApkPathChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(SignMessage::BrowseApk),
        ]
        .spacing(styles::SPACE_8),
        text("Keystore (optional)").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/keystore.jks", &state.keystore_path)
                .on_input(SignMessage::KeystorePathChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(SignMessage::BrowseKeystore),
        ]
        .spacing(styles::SPACE_8),
        text("Alias (optional)").size(styles::BODY_SIZE),
        text_input("Key alias", &state.alias)
            .on_input(SignMessage::AliasChanged)
            .width(Length::Fill),
        text("Keystore password (optional)").size(styles::BODY_SIZE),
        text_input("Keystore password", &state.keystore_pass)
            .secure(true)
            .on_input(SignMessage::KeystorePassChanged)
            .width(Length::Fill),
        text("Key password (optional)").size(styles::BODY_SIZE),
        text_input("Key password", &state.key_pass)
            .secure(true)
            .on_input(SignMessage::KeyPassChanged)
            .width(Length::Fill),
        text("If all signing fields are empty, ~/.android/debug.keystore is used")
            .size(styles::BODY_SIZE - 1.0)
            .style(text::secondary),
        button(text("Sign").size(styles::BODY_SIZE + 1.0))
            .width(Length::Fill)
            .padding([styles::SPACE_8, styles::SPACE_16])
            .on_press_maybe((!busy).then_some(SignMessage::Start)),
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
