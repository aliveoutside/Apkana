use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::config::ToolPaths;
use crate::ui::styles;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    JavaChanged(String),
    ApktoolJarChanged(String),
    ApksignerChanged(String),
    ZipalignChanged(String),
    AdbChanged(String),
    BrowseApktoolJar,
    Save,
    Cancel,
}

pub fn view(paths: &ToolPaths) -> Element<'_, SettingsMessage> {
    container(
        column![
            text("Settings").size(styles::SECTION_TITLE_SIZE),
            text("Java executable").size(styles::BODY_SIZE),
            text_input("java", &paths.java)
                .on_input(SettingsMessage::JavaChanged)
                .width(Length::Fill),
            text("apktool path (jar or executable)").size(styles::BODY_SIZE),
            row![
                text_input(
                    "/usr/bin/apktool or /path/to/apktool.jar",
                    &paths.apktool_jar
                )
                .on_input(SettingsMessage::ApktoolJarChanged)
                .width(Length::Fill),
                button("Browse")
                    .style(button::text)
                    .on_press(SettingsMessage::BrowseApktoolJar)
            ]
            .spacing(styles::SPACE_8),
            text("apksigner executable").size(styles::BODY_SIZE),
            text_input("apksigner", &paths.apksigner)
                .on_input(SettingsMessage::ApksignerChanged)
                .width(Length::Fill),
            text("zipalign executable").size(styles::BODY_SIZE),
            text_input("zipalign", &paths.zipalign)
                .on_input(SettingsMessage::ZipalignChanged)
                .width(Length::Fill),
            text("adb executable").size(styles::BODY_SIZE),
            text_input("adb", &paths.adb)
                .on_input(SettingsMessage::AdbChanged)
                .width(Length::Fill),
            row![
                iced::widget::Space::new().width(Length::Fill),
                button("Cancel")
                    .style(button::text)
                    .on_press(SettingsMessage::Cancel),
                button("Save").on_press(SettingsMessage::Save)
            ]
            .spacing(styles::SPACE_8)
        ]
        .spacing(styles::SPACE_8),
    )
    .padding(styles::SPACE_16)
    .style(container::rounded_box)
    .max_width(styles::MAX_SETTINGS_WIDTH)
    .width(Length::Fill)
    .into()
}
