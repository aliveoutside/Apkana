use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length, Theme};

use crate::app::{LogLevel, Message};
use crate::ui::common::{card, helper_text};
use crate::ui::styles;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub line: String,
}

pub fn view(logs: &[LogEntry]) -> Element<'_, Message> {
    let mut lines = column!().spacing(styles::SPACE_6);

    for entry in logs {
        let (prefix, color) = match entry.level {
            LogLevel::Info => ("INFO", Color::from_rgb8(225, 229, 236)),
            LogLevel::Warn => ("WARN", Color::from_rgb8(245, 194, 103)),
            LogLevel::Error => ("ERROR", Color::from_rgb8(244, 114, 114)),
        };
        lines = lines.push(
            column![
                text(prefix)
                    .size(styles::CAPTION_SIZE)
                    .style(move |_theme: &Theme| iced::widget::text::Style { color: Some(color) }),
                text(entry.line.clone())
                    .size(styles::BODY_SIZE)
                    .font(iced::Font::MONOSPACE)
                    .style(move |_theme: &Theme| iced::widget::text::Style { color: Some(color) }),
            ]
            .spacing(styles::SPACE_4),
        );
    }

    if logs.is_empty() {
        lines = lines.push(
            text("Tool output will appear here while commands run.")
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
                column![
                    text("Output").size(styles::SECTION_TITLE_SIZE),
                    helper_text("Execution logs, warnings, and failures from external Android tools."),
                ]
                .spacing(styles::SPACE_4),
                iced::widget::Space::new().width(Length::Fill),
                button("Copy")
                    .style(button::text)
                    .on_press(Message::CopyLogs),
                button("Clear")
                    .style(button::text)
                    .on_press(Message::ClearLogs),
            ]
            .spacing(styles::SPACE_6)
            .align_y(iced::Alignment::Center),
            container(scrollable(lines).height(Length::Fill))
                .height(Length::Fill)
                .padding(styles::SPACE_6),
        ]
        .spacing(styles::SPACE_10)
        .height(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}
