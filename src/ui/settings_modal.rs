use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::config::ToolPaths;
use crate::ui::common::{browse_button, field_label, form_shell, primary_action, secondary_button, section};
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
    let content = column![
        text("Settings").size(styles::PAGE_TITLE_SIZE),
        section(
            "Tool paths",
            column![
                field_label("Java executable"),
                text_input("java", &paths.java)
                    .size(styles::BODY_SIZE)
                    .on_input(SettingsMessage::JavaChanged)
                    .width(Length::Fill),
                field_label("apktool path (jar or executable)"),
                row![
                    text_input(
                        "/usr/bin/apktool or /path/to/apktool.jar",
                        &paths.apktool_jar
                    )
                    .size(styles::BODY_SIZE)
                    .on_input(SettingsMessage::ApktoolJarChanged)
                    .width(Length::Fill),
                    browse_button(SettingsMessage::BrowseApktoolJar),
                ]
                .spacing(styles::SPACE_6)
                .align_y(iced::Alignment::Center),
                field_label("apksigner executable"),
                text_input("apksigner", &paths.apksigner)
                    .size(styles::BODY_SIZE)
                    .on_input(SettingsMessage::ApksignerChanged)
                    .width(Length::Fill),
                field_label("zipalign executable"),
                text_input("zipalign", &paths.zipalign)
                    .size(styles::BODY_SIZE)
                    .on_input(SettingsMessage::ZipalignChanged)
                    .width(Length::Fill),
                field_label("adb executable"),
                text_input("adb", &paths.adb)
                    .size(styles::BODY_SIZE)
                    .on_input(SettingsMessage::AdbChanged)
                    .width(Length::Fill),
            ]
            .spacing(styles::SPACE_6),
        ),
        row![
            iced::widget::Space::new().width(Length::Fill),
            button(text("Cancel").size(styles::BODY_SIZE))
                .style(secondary_button)
                .padding([styles::SPACE_6, styles::SPACE_16])
                .on_press(SettingsMessage::Cancel),
            button(text("Save").size(styles::BODY_SIZE))
                .style(primary_action)
                .padding([styles::SPACE_6, styles::SPACE_16])
                .on_press(SettingsMessage::Save)
        ]
        .spacing(styles::SPACE_8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(styles::SECTION_GAP);

    form_shell(content).max_width(styles::MAX_SETTINGS_WIDTH).into()
}
