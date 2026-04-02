use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

use crate::config::ToolPaths;
use crate::ui::common::{field_label, form_shell, helper_text, primary_action, section};
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
        helper_text("Configure the external Android tools Apkana calls at runtime."),
        section(
            "Tool paths",
            Some("These can point to binaries on PATH or absolute file paths."),
            column![
                field_label("Java executable"),
                text_input("java", &paths.java)
                    .on_input(SettingsMessage::JavaChanged)
                    .width(Length::Fill),
                field_label("apktool path (jar or executable)"),
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
                field_label("apksigner executable"),
                text_input("apksigner", &paths.apksigner)
                    .on_input(SettingsMessage::ApksignerChanged)
                    .width(Length::Fill),
                field_label("zipalign executable"),
                text_input("zipalign", &paths.zipalign)
                    .on_input(SettingsMessage::ZipalignChanged)
                    .width(Length::Fill),
                field_label("adb executable"),
                text_input("adb", &paths.adb)
                    .on_input(SettingsMessage::AdbChanged)
                    .width(Length::Fill),
            ]
            .spacing(styles::SPACE_8),
        ),
        row![
            iced::widget::Space::new().width(Length::Fill),
            button("Cancel")
                .style(button::text)
                .on_press(SettingsMessage::Cancel),
            button("Save").style(primary_action).padding([styles::SPACE_6, styles::SPACE_12]).on_press(SettingsMessage::Save)
        ]
        .spacing(styles::SPACE_8),
    ]
    .spacing(styles::SPACE_10);

    form_shell(content).max_width(styles::MAX_SETTINGS_WIDTH).into()
}
