use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{field_label, form_shell, helper_text, primary_action, section};
use crate::ui::styles;

#[derive(Debug, Clone, Default)]
pub struct InstallTabState {
    pub apk_path: String,
}

#[derive(Debug, Clone)]
pub enum InstallMessage {
    ApkPathChanged(String),
    BrowseApk,
    Start,
}

pub fn view(state: &InstallTabState, busy: bool) -> Element<'_, InstallMessage> {
    let content = column![
        text("Install APK").size(styles::PAGE_TITLE_SIZE),
        helper_text("Install an APK on a connected Android device with `adb install -r`."),
        section(
            "Package",
            Some("Choose the APK file to send to the currently connected device."),
            column![
                field_label("APK file"),
                row![
                    text_input("/path/to/app.apk", &state.apk_path)
                        .on_input(InstallMessage::ApkPathChanged)
                        .width(Length::Fill),
                    button("Browse")
                        .style(button::text)
                        .on_press(InstallMessage::BrowseApk),
                ]
                .spacing(styles::SPACE_8),
            ],
        ),
        button(text("Install APK").size(styles::PRIMARY_BUTTON_TEXT_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_10, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(InstallMessage::Start)),
    ]
    .spacing(styles::SPACE_10);

    form_shell(content).into()
}
