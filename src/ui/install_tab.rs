use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::ui::common::{browse_button, field_label, form_shell, primary_action, section};
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
        section(
            "Package",
            column![
                field_label("APK file"),
                row![
                    text_input("/path/to/app.apk", &state.apk_path)
                        .size(styles::BODY_SIZE)
                        .on_input(InstallMessage::ApkPathChanged)
                        .width(Length::Fill),
                    browse_button(InstallMessage::BrowseApk),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
            ],
        ),
        button(text("Install APK").size(styles::BODY_SIZE))
            .style(primary_action)
            .width(Length::Fill)
            .padding([styles::SPACE_6, styles::SPACE_12])
            .on_press_maybe((!busy).then_some(InstallMessage::Start)),
    ]
    .spacing(styles::SECTION_GAP);

    form_shell(content).into()
}
