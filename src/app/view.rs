use eframe::egui;

use super::{ApkanaApp, LogLevel, Message};
use crate::config::ThemeMode;
use crate::ui::common::{self, Palette};
use crate::ui::tabs::MainTab;
use crate::ui::{build_tab, decode_tab, install_tab, merge_tab, settings_modal, sign_tab};

impl ApkanaApp {
    pub(super) fn render_frame(&mut self, ctx: &egui::Context) {
        let p = common::palette(ctx.style().visuals.dark_mode);
        let mut messages = Vec::new();

        // Persistent navigation keeps each operation one click away.
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(228.0)
            .frame(
                egui::Frame::new()
                    .fill(p.sidebar)
                    .inner_margin(egui::Margin::symmetric(17, 20))
                    .stroke(egui::Stroke::new(1.0, p.border)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::Frame::new()
                        .fill(p.accent)
                        .corner_radius(egui::CornerRadius::same(11))
                        .inner_margin(egui::Margin::symmetric(12, 9))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("A")
                                    .size(20.0)
                                    .strong()
                                    .color(p.accent_text),
                            );
                        });
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("APKANA")
                                .size(19.0)
                                .strong()
                                .color(p.text),
                        );
                        ui.label(
                            egui::RichText::new("ANDROID TOOLKIT")
                                .size(9.0)
                                .strong()
                                .color(p.muted),
                        );
                    });
                });

                ui.add_space(42.0);
                ui.label(
                    egui::RichText::new("WORKSPACE")
                        .size(10.0)
                        .strong()
                        .color(p.muted),
                );
                ui.add_space(11.0);

                for (index, tab) in MainTab::ALL.into_iter().enumerate() {
                    let selected = self.active_tab == tab;
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 46.0),
                        egui::Sense::click(),
                    );
                    let background = if selected {
                        p.selected
                    } else if response.hovered() {
                        p.surface_hover
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::same(9),
                        background,
                    );
                    if selected {
                        let indicator = egui::Rect::from_min_size(
                            rect.left_center() + egui::vec2(0.0, -11.0),
                            egui::vec2(3.0, 22.0),
                        );
                        ui.painter().rect_filled(
                            indicator,
                            egui::CornerRadius::same(2),
                            p.accent,
                        );
                    }
                    ui.painter().text(
                        rect.left_center() + egui::vec2(16.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        format!("{:02}", index + 1),
                        egui::FontId::monospace(11.0),
                        if selected { p.accent } else { p.muted },
                    );
                    ui.painter().text(
                        rect.left_center() + egui::vec2(50.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        tab.title(),
                        egui::FontId::proportional(14.0),
                        if selected { p.text } else { p.muted },
                    );
                    if response.clicked() {
                        messages.push(Message::TabSelected(tab));
                    }
                    ui.add_space(5.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(7.0);
                    ui.label(
                        egui::RichText::new("v0.1  /  DESKTOP")
                            .size(10.0)
                            .color(p.muted),
                    );
                    ui.add_space(11.0);
                    let button = egui::Button::new(
                        egui::RichText::new("Preferences")
                            .size(13.0)
                            .color(p.text),
                    )
                    .fill(p.surface)
                    .stroke(egui::Stroke::new(1.0, p.border))
                    .corner_radius(egui::CornerRadius::same(9));
                    if ui
                        .add_enabled_ui(!self.busy, |ui| {
                            ui.add_sized([ui.available_width(), 40.0], button)
                        })
                        .inner
                        .clicked()
                    {
                        messages.push(Message::SettingsPressed);
                    }
                    ui.add_space(9.0);
                    ui.separator();
                });
            });

        let mut copy_logs = false;
        let mut clear_logs = false;
        egui::TopBottomPanel::bottom("output")
            .resizable(true)
            .default_height(205.0)
            .min_height(125.0)
            .frame(
                egui::Frame::new()
                    .fill(p.sidebar)
                    .inner_margin(egui::Margin::symmetric(23, 12))
                    .stroke(egui::Stroke::new(1.0, p.border)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("ACTIVITY")
                            .size(11.0)
                            .strong()
                            .color(p.text),
                    );
                    ui.add_space(8.0);
                    if self.busy {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("IN PROGRESS")
                                .size(10.0)
                                .strong()
                                .color(p.accent),
                        );
                    } else if !self.status_message.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.status_message)
                                .size(12.0)
                                .color(p.muted),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Ready for the next task")
                                .size(12.0)
                                .color(p.muted),
                        );
                    }
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            clear_logs = ui.small_button("Clear").clicked();
                            copy_logs = ui.small_button("Copy logs").clicked();
                        },
                    );
                });

                if self.busy {
                    ui.add_space(8.0);
                    ui.add(
                        egui::ProgressBar::new((self.progress_value / 100.0).clamp(0.0, 1.0))
                            .fill(p.accent)
                            .show_percentage(),
                    );
                }
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(5.0);
                egui::ScrollArea::vertical()
                    .id_salt("activity_history")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.logs.is_empty() {
                            ui.add_space(14.0);
                            ui.label(
                                egui::RichText::new("Command output and status updates appear here.")
                                    .size(12.5)
                                    .color(p.muted),
                            );
                        }
                        for entry in &self.logs {
                            let (tag, color) = match entry.level {
                                LogLevel::Info => ("INFO", p.muted),
                                LogLevel::Warn => ("WARN", egui::Color32::from_rgb(224, 177, 100)),
                                LogLevel::Error => ("ERROR", egui::Color32::from_rgb(234, 113, 107)),
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(tag)
                                        .monospace()
                                        .size(11.0)
                                        .strong()
                                        .color(color),
                                );
                                ui.label(
                                    egui::RichText::new(&entry.line)
                                        .monospace()
                                        .size(12.0)
                                        .color(p.text),
                                );
                            });
                        }
                    });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(p.canvas)
                    .inner_margin(egui::Margin::symmetric(27, 25)),
            )
            .show(ctx, |ui| {
                let (title, description) = match self.active_tab {
                    MainTab::DecodeBuild => (
                        "Decode & Build",
                        "Disassemble, edit, and rebuild Android packages.",
                    ),
                    MainTab::Sign => (
                        "Sign APK",
                        "Prepare a package with a signing certificate.",
                    ),
                    MainTab::Merge => (
                        "Merge packages",
                        "Turn compatible split archives into a single APK.",
                    ),
                    MainTab::Install => (
                        "Install on device",
                        "Push an Android package to a connected device.",
                    ),
                };
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .size(28.0)
                                .strong()
                                .color(p.text),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(description)
                                .size(13.0)
                                .color(p.muted),
                        );
                    });
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new("APK WORKSPACE")
                                    .size(10.0)
                                    .strong()
                                    .color(p.muted),
                            );
                        },
                    );
                });
                ui.add_space(25.0);

                ui.add_enabled_ui(!self.busy, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("workflow_content")
                        .show(ui, |ui| {
                            match self.active_tab {
                                MainTab::DecodeBuild => {
                                    if ui.available_width() > 820.0 {
                                        ui.columns(2, |cols| {
                                            cols[0].spacing_mut().item_spacing.y = 12.0;
                                            cols[1].spacing_mut().item_spacing.y = 12.0;
                                            messages.extend(
                                                decode_tab::show(
                                                    &mut cols[0],
                                                    &self.decode,
                                                    self.busy,
                                                )
                                                .into_iter()
                                                .map(Message::Decode),
                                            );
                                            messages.extend(
                                                build_tab::show(
                                                    &mut cols[1],
                                                    &self.build,
                                                    self.busy,
                                                )
                                                .into_iter()
                                                .map(Message::Build),
                                            );
                                        });
                                    } else {
                                        ui.set_max_width(780.0);
                                        messages.extend(
                                            decode_tab::show(ui, &self.decode, self.busy)
                                                .into_iter()
                                                .map(Message::Decode),
                                        );
                                        ui.add_space(15.0);
                                        messages.extend(
                                            build_tab::show(ui, &self.build, self.busy)
                                                .into_iter()
                                                .map(Message::Build),
                                        );
                                    }
                                }
                                MainTab::Sign => {
                                    ui.set_max_width(760.0);
                                    messages.extend(
                                        sign_tab::show(ui, &self.sign, self.busy)
                                            .into_iter()
                                            .map(Message::Sign),
                                    );
                                }
                                MainTab::Merge => {
                                    ui.set_max_width(760.0);
                                    messages.extend(
                                        merge_tab::show(ui, &self.merge, self.busy)
                                            .into_iter()
                                            .map(Message::Merge),
                                    );
                                }
                                MainTab::Install => {
                                    ui.set_max_width(760.0);
                                    messages.extend(
                                        install_tab::show(ui, &self.install, self.busy)
                                            .into_iter()
                                            .map(Message::Install),
                                    );
                                }
                            }
                            ui.add_space(20.0);
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
            egui::Window::new("Preferences")
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(p.surface)
                        .stroke(egui::Stroke::new(1.0, p.border))
                        .corner_radius(egui::CornerRadius::same(13))
                        .inner_margin(egui::Margin::same(20)),
                )
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
        let dark = !matches!(self.config.theme_mode, ThemeMode::Light);
        let p: Palette = common::palette(dark);
        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = p.canvas;
        visuals.window_fill = p.surface;
        visuals.window_stroke = egui::Stroke::new(1.0, p.border);
        visuals.window_corner_radius = egui::CornerRadius::same(13);
        visuals.extreme_bg_color = p.field;
        visuals.faint_bg_color = p.surface_hover;
        visuals.code_bg_color = p.field;
        visuals.override_text_color = Some(p.text);
        visuals.weak_text_color = Some(p.muted);
        visuals.selection.bg_fill = p.selected;
        visuals.selection.stroke = egui::Stroke::new(1.5, p.accent);
        visuals.hyperlink_color = p.accent;
        visuals.widgets.noninteractive.bg_fill = p.surface;
        visuals.widgets.noninteractive.fg_stroke.color = p.text;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.border);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = egui::CornerRadius::same(8);
            widget.bg_stroke = egui::Stroke::new(1.0, p.border);
            widget.fg_stroke.color = p.text;
        }
        visuals.widgets.inactive.bg_fill = p.field;
        visuals.widgets.inactive.weak_bg_fill = p.surface;
        visuals.widgets.hovered.bg_fill = p.surface_hover;
        visuals.widgets.hovered.weak_bg_fill = p.surface_hover;
        visuals.widgets.active.bg_fill = p.selected;
        visuals.widgets.active.weak_bg_fill = p.selected;
        ctx.set_visuals(visuals);

        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(9.0, 8.0);
            style.spacing.button_padding = egui::vec2(11.0, 7.0);
            style.spacing.interact_size = egui::vec2(40.0, 31.0);
            // Keep the existing +1 point accessibility bump.
            for font in style.text_styles.values_mut() {
                font.size += 1.0;
            }
        });
    }
}
