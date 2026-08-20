use iced::widget::{
    button, column, container, pane_grid, row, scrollable, stack, text, Space,
};
use iced::{Background, Color, Element, Length, Theme};

use crate::ui::build_tab;
use crate::ui::common::{card, secondary_button};
use crate::ui::decode_tab;
use crate::ui::install_tab;
use crate::ui::log_panel;
use crate::ui::merge_tab;
use crate::ui::settings_modal;
use crate::ui::sign_tab;
use crate::ui::styles;
use crate::ui::tabs::{self, MainTab};

use super::{ApkanaApp, Message, ShellPane};

impl ApkanaApp {
    pub fn view(&self) -> Element<'_, Message> {
        let tabs = tabs::view(
            self.active_tab,
            Message::TabSelected,
            Some(
                button(text("Settings").size(styles::BODY_SIZE))
                    .style(secondary_button)
                    .padding([styles::SPACE_5, styles::SPACE_10])
                    .on_press(Message::SettingsPressed)
                    .into(),
            ),
        );

        let top_bar = card(column![tabs]);

        let pane_grid = pane_grid(&self.output_panes, |_, pane, _| {
            let content: Element<'_, Message> = match pane {
                ShellPane::Content => {
                    let workflow_content = match self.active_tab {
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
                        MainTab::Merge => {
                            merge_tab::view(&self.merge, self.busy).map(Message::Merge)
                        }
                        MainTab::Install => {
                            install_tab::view(&self.install, self.busy).map(Message::Install)
                        }
                    };

                    container(
                        scrollable(workflow_content)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                }
                ShellPane::Output => log_panel::view(&self.logs),
            };

            pane_grid::Content::new(content)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(8)
        .on_resize(12, Message::OutputResized);

        let body = column![
            top_bar,
            container(pane_grid)
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .spacing(styles::SPACE_12)
        .padding(styles::SPACE_12)
        .height(Length::Fill);

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
            .padding(styles::SPACE_16)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

            stack![base, backdrop, modal].into()
        } else {
            base.into()
        }
    }
}
