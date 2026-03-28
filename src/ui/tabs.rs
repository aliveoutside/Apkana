use iced::widget::{Space, button, column, container, row, rule, text};
use iced::{Element, Length};

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
) -> Element<'a, Message> {
    MainTab::ALL
        .into_iter()
        .fold(row!().spacing(styles::SPACE_8), |row, tab| {
            let is_active = tab == active;

            let indicator = if is_active {
                rule::horizontal(2)
            } else {
                rule::horizontal(1)
            };

            let tab_button = button(
                column![
                    text(tab.title()).size(styles::BODY_SIZE),
                    container(indicator)
                        .width(Length::Fill)
                        .height(Length::Fixed(if is_active { 3.0 } else { 2.0 }))
                ]
                .spacing(styles::SPACE_4),
            )
            .style(button::text)
            .on_press(on_select(tab));

            row.push(
                container(tab_button)
                    .padding([styles::SPACE_4, styles::SPACE_8])
                    .width(Length::Shrink),
            )
        })
        .push(Space::new().width(Length::Fill))
        .into()
}
