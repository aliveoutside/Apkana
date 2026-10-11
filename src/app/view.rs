use eframe::egui;

use super::{ApkanaApp, LogLevel, Message};
use crate::config::ThemeMode;
use crate::ui::tabs::MainTab;
use crate::ui::{build_tab, decode_tab, install_tab, merge_tab, settings_modal, sign_tab};

impl ApkanaApp {
    pub(super) fn render_frame(&mut self, ctx: &egui::Context) {
        let mut messages = Vec::new();

        egui::TopBottomPanel::top("navigation").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading("Apkana");
                ui.separator();
                for tab in MainTab::ALL {
                    if ui
                        .selectable_label(self.active_tab == tab, tab.title())
                        .clicked()
                    {
                        messages.push(Message::TabSelected(tab));
                    }
                }
                ui.separator();
                if ui
                    .add_enabled(!self.busy, egui::Button::new("Settings"))
                    .clicked()
                {
                    messages.push(Message::SettingsPressed);
                }
            });
            ui.add_space(6.0);
        });

        let mut copy_logs = false;
        let mut clear_logs = false;
        egui::TopBottomPanel::bottom("output")
            .resizable(true)
            .default_height(210.0)
            .min_height(110.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OUTPUT").strong());
                    if !self.status_message.is_empty() {
                        ui.separator();
                        ui.label(&self.status_message);
                    }
                    if self.busy {
                        ui.spinner();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        clear_logs = ui.button("Clear").clicked();
                        copy_logs = ui.button("Copy").clicked();
                    });
                });
                if self.busy {
                    ui.add(
                        egui::ProgressBar::new((self.progress_value / 100.0).clamp(0.0, 1.0))
                            .show_percentage(),
                    );
                }
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.logs.is_empty() {
                            ui.label("Tool output will appear here...");
                        }
                        for entry in &self.logs {
                            let (tag, color) = match entry.level {
                                LogLevel::Info => ("INFO", egui::Color32::LIGHT_GRAY),
                                LogLevel::Warn => ("WARN", egui::Color32::YELLOW),
                                LogLevel::Error => ("ERROR", egui::Color32::LIGHT_RED),
                            };
                            ui.label(
                                egui::RichText::new(format!("[{tag}] {}", entry.line))
                                    .monospace()
                                    .color(color),
                            );
                        }
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.busy, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    match self.active_tab {
                        MainTab::DecodeBuild => {
                            ui.columns(2, |cols| {
                                messages.extend(
                                    decode_tab::show(&mut cols[0], &self.decode, self.busy)
                                        .into_iter()
                                        .map(Message::Decode),
                                );
                                messages.extend(
                                    build_tab::show(&mut cols[1], &self.build, self.busy)
                                        .into_iter()
                                        .map(Message::Build),
                                );
                            });
                        }
                        MainTab::Sign => {
                            messages.extend(
                                sign_tab::show(ui, &self.sign, self.busy)
                                    .into_iter()
                                    .map(Message::Sign),
                            );
                        }
                        MainTab::Merge => {
                            messages.extend(
                                merge_tab::show(ui, &self.merge, self.busy)
                                    .into_iter()
                                    .map(Message::Merge),
                            );
                        }
                        MainTab::Install => {
                            messages.extend(
                                install_tab::show(ui, &self.install, self.busy)
                                    .into_iter()
                                    .map(Message::Install),
                            );
                        }
                    }
                });
            });
        });

        if copy_logs {
            ctx.copy_text(self.logs_for_clipboard());
        }
        if clear_logs {
            messages.push(Message::ClearLogs);
        }
        for message in messages {
            self.dispatch(message);
        }

        if self.show_settings {
            let draft = self
                .settings_draft
                .clone()
                .unwrap_or_else(|| self.config.tools.clone());
            let mut settings_events = Vec::new();
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.add_enabled_ui(!self.busy, |ui| {
                        settings_events = settings_modal::show(ui, &draft);
                    });
                });
            for event in settings_events {
                self.dispatch(Message::Settings(event));
            }
        }
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        match self.config.theme_mode {
            ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
            ThemeMode::Dark | ThemeMode::System => ctx.set_visuals(egui::Visuals::dark()),
        }

        // Increase all built-in text styles by one point throughout the UI.
        ctx.style_mut(|style| {
            for font in style.text_styles.values_mut() {
                font.size += 1.0;
            }
        });
    }
}
