use iced::widget::{button, column, container, text, Column, Container};
use iced::{Element, Length, Theme};

use crate::ui::styles;

/// Style for primary action buttons (Decode, Build, Sign, etc.) that matches
/// the design system's border radius and uses the theme's primary palette.
pub fn primary_action(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = styles::PANEL_RADIUS.into();
    style
}

pub fn card<'a, Message: 'a>(content: Column<'a, Message>) -> Container<'a, Message> {
    container(content)
        .padding(styles::CARD_PADDING)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                border: iced::Border {
                    radius: styles::CARD_RADIUS.into(),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                ..Default::default()
            }
        })
}

pub fn section_title<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(styles::SECTION_TITLE_SIZE)
        .style(text::primary)
        .into()
}

pub fn helper_text<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(styles::SUPPORTING_TEXT_SIZE)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::text::Style {
                color: Some(palette.background.base.text.scale_alpha(0.86)),
            }
        })
        .into()
}

pub fn field_label<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(styles::LABEL_SIZE)
        .style(text::primary)
        .into()
}

pub fn section<'a, Message: 'a>(
    title: &'a str,
    description: Option<&'a str>,
    content: Column<'a, Message>,
) -> Column<'a, Message> {
    let mut column = column![section_title(title)].spacing(styles::SPACE_6);

    if let Some(description) = description {
        column = column.push(helper_text(description));
    }

    column.push(content.spacing(styles::SPACE_6))
}

pub fn form_shell<'a, Message: 'a>(content: Column<'a, Message>) -> Container<'a, Message> {
    container(card(content.spacing(styles::SECTION_GAP)))
        .width(Length::Fill)
        .max_width(styles::MAX_FORM_WIDTH)
        .center_x(Length::Fill)
}
