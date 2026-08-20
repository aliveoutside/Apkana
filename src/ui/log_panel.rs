use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length, Theme};

use crate::app::{LogLevel, Message};
use crate::ui::common::{card, secondary_button};
use crate::ui::styles;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub line: String,
}

pub fn view(logs: &[LogEntry]) -> Element<'_, Message> {
    let mut lines = column!().spacing(styles::SPACE_4).width(Length::Fill);

    for entry in logs {
        let (prefix, color) = match entry.level {
            LogLevel::Info => ("[INFO]", Color::from_rgb8(225, 229, 236)),
            LogLevel::Warn => ("[WARN]", Color::from_rgb8(245, 194, 103)),
            LogLevel::Error => ("[ERROR]", Color::from_rgb8(244, 114, 114)),
        };
        lines = lines.push(
            text(format!("{prefix} {}", entry.line))
                .size(styles::BODY_SIZE)
                .font(iced::Font::MONOSPACE)
                .style(move |_theme: &Theme| iced::widget::text::Style { color: Some(color) }),
        );
    }

    if logs.is_empty() {
        lines = lines.push(
            text("Tool output will appear here...")
                .size(styles::BODY_SIZE)
                .font(iced::Font::MONOSPACE)
                .style(|_theme: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(158, 168, 186)),
                }),
        );
    }

    card(
        column![
            row![
                text("OUTPUT")
                    .size(styles::LABEL_SIZE)
                    .style(text::secondary),
                iced::widget::Space::new().width(Length::Fill),
                button(text("Copy").size(styles::LABEL_SIZE))
                    .style(secondary_button)
                    .padding([styles::SPACE_4, styles::SPACE_8])
                    .on_press(Message::CopyLogs),
                button(text("Clear").size(styles::LABEL_SIZE))
                    .style(secondary_button)
                    .padding([styles::SPACE_4, styles::SPACE_8])
                    .on_press(Message::ClearLogs),
            ]
            .spacing(styles::SPACE_6)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
            container(
                scrollable(lines)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(styles::SPACE_4),
        ]
        .spacing(styles::SPACE_8)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
