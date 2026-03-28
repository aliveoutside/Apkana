use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

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
    let card = column![
        text("Install APK").size(styles::SECTION_TITLE_SIZE),
        text("Install an APK on a connected device via adb")
            .size(styles::BODY_SIZE)
            .style(text::secondary),
        text("APK file").size(styles::BODY_SIZE),
        row![
            text_input("/path/to/app.apk", &state.apk_path)
                .on_input(InstallMessage::ApkPathChanged)
                .width(Length::Fill),
            button("Browse")
                .style(button::text)
                .on_press(InstallMessage::BrowseApk),
        ]
        .spacing(styles::SPACE_8),
        button(text("Install").size(styles::BODY_SIZE + 1.0))
            .width(Length::Fill)
            .padding([styles::SPACE_8, styles::SPACE_16])
            .on_press_maybe((!busy).then_some(InstallMessage::Start)),
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
