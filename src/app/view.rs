use iced::widget::{
    button, column, container, progress_bar, row, rule, scrollable, stack, text, Space,
};
use iced::{Background, Color, Element, Length, Theme};

use crate::ui::build_tab;
use crate::ui::decode_tab;
use crate::ui::install_tab;
use crate::ui::log_panel;
use crate::ui::merge_tab;
use crate::ui::settings_modal;
use crate::ui::sign_tab;
use crate::ui::styles;
use crate::ui::tabs::{self, MainTab};

use super::{ApkanaApp, Message};

impl ApkanaApp {
    pub fn view(&self) -> Element<'_, Message> {
        let tabs = tabs::view(self.active_tab, Message::TabSelected);

        let content = match self.active_tab {
            MainTab::DecodeBuild => row![
                container(decode_tab::view(&self.decode, self.busy).map(Message::Decode))
                    .width(Length::FillPortion(1)),
                container(build_tab::view(&self.build, self.busy).map(Message::Build))
                    .width(Length::FillPortion(1)),
            ]
            .spacing(styles::SPACE_16)
            .align_y(iced::Alignment::Start)
            .into(),
            MainTab::Sign => sign_tab::view(&self.sign, self.busy).map(Message::Sign),
            MainTab::Merge => merge_tab::view(&self.merge, self.busy).map(Message::Merge),
            MainTab::Install => install_tab::view(&self.install, self.busy).map(Message::Install),
        };

        let top = row![
            text("Apkana").size(styles::TITLE_SIZE),
            Space::new().width(Length::Fill),
            button("Settings")
                .style(button::text)
                .on_press(Message::SettingsPressed)
        ]
        .spacing(styles::SPACE_16)
        .align_y(iced::Alignment::Center);

        let mut body = column![top, rule::horizontal(1), tabs,]
            .spacing(styles::SPACE_16)
            .padding(styles::SPACE_16)
            .height(Length::Fill);

        if self.busy {
            body = body.push(
                row![
                    text("Running...").size(styles::BODY_SIZE),
                    container(progress_bar(0.0..=100.0, self.progress_value)).width(Length::Fill),
                ]
                .spacing(styles::SPACE_8)
                .align_y(iced::Alignment::Center),
            );
        }

        if !self.status_message.is_empty() {
            body = body.push(
                text(self.status_message.clone())
                    .size(styles::BODY_SIZE)
                    .style(text::primary),
            );
        }

        body = body.push(
            container(
                scrollable(container(content).padding(styles::SPACE_16)).height(Length::Fill),
            )
            .height(Length::FillPortion(3)),
        );

        body = body.push(container(log_panel::view(&self.logs)).height(Length::FillPortion(2)));

        let base = container(body).width(Length::Fill).height(Length::Fill);

        if self.show_settings {
            let backdrop = container(Space::new().width(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                    ..Default::default()
                });

            let modal = container(
                settings_modal::view(self.settings_draft.as_ref().unwrap_or(&self.config.tools))
                    .map(Message::Settings),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(styles::SPACE_24)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

            stack![base, backdrop, modal].into()
        } else {
            base.into()
        }
    }
}
