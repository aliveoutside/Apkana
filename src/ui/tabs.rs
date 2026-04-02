use iced::widget::{Space, button, container, row, text};
use iced::{Element, Length, Theme};

use crate::ui::styles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    DecodeBuild,
    Sign,
    Merge,
    Install,
}

impl MainTab {
    pub const ALL: [Self; 4] = [Self::DecodeBuild, Self::Sign, Self::Merge, Self::Install];

    pub fn title(self) -> &'static str {
        match self {
            Self::DecodeBuild => "Decode + Build",
            Self::Sign => "Sign",
            Self::Merge => "Merge",
            Self::Install => "Install",
        }
    }
}

pub fn view<'a, Message: Clone + 'a>(
    active: MainTab,
    on_select: impl Fn(MainTab) -> Message + 'a,
    trailing_action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let row = MainTab::ALL
        .into_iter()
        .fold(
            row!().spacing(styles::SPACE_8).align_y(iced::Alignment::Center),
            |row, tab| {
            let is_active = tab == active;

            let tab_button = button(
                text(tab.title())
                    .size(styles::BODY_SIZE)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        let color = if is_active {
                            palette.primary.base.text
                        } else {
                            palette.background.strong.text
                        };
                        iced::widget::text::Style { color: Some(color) }
                    }),
            )
            .padding([styles::SPACE_6, styles::SPACE_12])
            .style(move |theme: &Theme, status| {
                let palette = theme.extended_palette();
                let mut style = iced::widget::button::text(theme, status);
                style.border.radius = styles::PANEL_RADIUS.into();
                style.border.width = 1.0;
                style.border.color = if is_active {
                    palette.primary.strong.color
                } else {
                    palette.background.strong.color
                };
                style.background = Some(iced::Background::Color(if is_active {
                    palette.primary.weak.color
                } else {
                    palette.background.weak.color
                }));
                style
            })
            .on_press(on_select(tab));

            row.push(container(tab_button).width(Length::Shrink))
        });

    let row = row.push(Space::new().width(Length::Fill));

    match trailing_action {
        Some(action) => row.push(action).into(),
        None => row.into(),
    }
}
