use iced::widget::{button, column, container, text, Button, Column, Container};
use iced::{Element, Length, Theme};

use crate::ui::styles;

pub fn primary_action(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = styles::CONTROL_RADIUS.into();
    style
}

pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let mut style = button::secondary(theme, status);
    style.border.radius = styles::CONTROL_RADIUS.into();
    style.border.width = 1.0;

    match status {
        button::Status::Active => {
            style.background = Some(iced::Background::Color(palette.background.strong.color));
            style.text_color = palette.background.base.text;
            style.border.color = palette.background.strong.text.scale_alpha(0.2);
        }
        button::Status::Hovered => {
            style.background = Some(iced::Background::Color(palette.primary.weak.color));
            style.text_color = palette.primary.base.text;
            style.border.color = palette.primary.strong.color;
        }
        button::Status::Pressed => {
            style.background = Some(iced::Background::Color(palette.primary.strong.color));
            style.text_color = palette.primary.base.text;
            style.border.color = palette.primary.strong.color;
        }
        button::Status::Disabled => {
            style.background = Some(iced::Background::Color(palette.background.weak.color));
            style.text_color = palette.background.strong.text.scale_alpha(0.5);
            style.border.color = palette.background.strong.color;
        }
    }

    style
}

pub fn browse_button<'a, Message: 'a + Clone>(on_press: Message) -> Button<'a, Message> {
    button(text("Browse").size(styles::LABEL_SIZE))
        .style(secondary_button)
        .padding([styles::SPACE_5, styles::SPACE_10])
        .on_press(on_press)
}

pub fn card<'a, Message: 'a>(content: Column<'a, Message>) -> Container<'a, Message> {
    container(content)
        .width(Length::Fill)
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

pub fn field_label<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    text(label)
        .size(styles::LABEL_SIZE)
        .style(text::secondary)
        .into()
}

pub fn section<'a, Message: 'a>(
    title: &'a str,
    content: Column<'a, Message>,
) -> Column<'a, Message> {
    column![section_title(title), content.spacing(styles::SPACE_6)].spacing(styles::SPACE_8)
}

pub fn form_shell<'a, Message: 'a>(content: Column<'a, Message>) -> Container<'a, Message> {
    container(card(content.spacing(styles::SECTION_GAP)))
        .width(Length::Fill)
        .max_width(styles::MAX_FORM_WIDTH)
        .center_x(Length::Fill)
}
