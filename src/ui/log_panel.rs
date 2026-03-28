use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length, Theme};

use crate::app::{LogLevel, Message};
use crate::ui::styles;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub line: String,
}

pub fn view(logs: &[LogEntry]) -> Element<'_, Message> {
    let mut lines = column!().spacing(styles::SPACE_4);

    for entry in logs {
        let (prefix, color) = match entry.level {
            LogLevel::Info => ("[INFO]", Color::from_rgb8(225, 229, 236)),
            LogLevel::Warn => ("[WARN]", Color::from_rgb8(245, 194, 103)),
            LogLevel::Error => ("[ERR ]", Color::from_rgb8(244, 114, 114)),
        };
        lines = lines.push(
            text(format!("{prefix} {}", entry.line))
                .size(styles::BODY_SIZE)
                .style(move |_theme: &Theme| iced::widget::text::Style { color: Some(color) })
                .font(iced::Font::MONOSPACE),
        );
    }

    if logs.is_empty() {
        lines = lines.push(
            text("[INFO] Output from tools will appear here.")
                .size(styles::BODY_SIZE)
                .font(iced::Font::MONOSPACE)
                .style(|_theme: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(158, 168, 186)),
                }),
        );
    }

    let header = row![
        text("Output")
            .size(styles::SECTION_TITLE_SIZE)
            .style(text::primary),
        iced::widget::Space::new().width(Length::Fill),
        button("Copy")
            .style(button::text)
            .on_press(Message::CopyLogs),
        button("Clear")
            .style(button::text)
            .on_press(Message::ClearLogs)
    ]
    .spacing(styles::SPACE_8)
    .align_y(iced::Alignment::Center);

    container(
        column![
            header,
            container(scrollable(lines).height(Length::Fill))
                .height(Length::Fill)
                .padding(styles::SPACE_8),
        ]
        .spacing(styles::SPACE_8)
        .height(Length::Fill),
    )
    .padding(styles::SPACE_16)
    .style(container::rounded_box)
    .height(Length::Fill)
    .into()
}
