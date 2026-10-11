//! Shared egui form controls.
use eframe::egui;

pub fn card(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width((ui.available_width() - 16.0).max(0.0));
        ui.heading(title);
        ui.add_space(6.0);
        content(ui);
    });
}

pub fn field(ui: &mut egui::Ui, label: &str, hint: &str, value: &str) -> Option<String> {
    edit(ui, label, hint, value, false)
}

pub fn secret(ui: &mut egui::Ui, label: &str, hint: &str, value: &str) -> Option<String> {
    edit(ui, label, hint, value, true)
}

fn edit(ui: &mut egui::Ui, label: &str, hint: &str, value: &str, secret: bool) -> Option<String> {
    ui.label(label);
    let mut draft = value.to_owned();
    let response = ui.add(
        egui::TextEdit::singleline(&mut draft)
            .hint_text(hint)
            .password(secret)
            .desired_width(f32::INFINITY),
    );
    response.changed().then_some(draft)
}

pub fn path(ui: &mut egui::Ui, label: &str, hint: &str, value: &str) -> (Option<String>, bool) {
    ui.label(label);
    let mut draft = value.to_owned();
    let mut changed = false;
    let mut browse = false;
    ui.horizontal(|ui| {
        changed = ui.add(
            egui::TextEdit::singleline(&mut draft)
                .hint_text(hint)
                .desired_width((ui.available_width() - 84.0).max(100.0)),
        ).changed();
        browse = ui.button("Browse").clicked();
    });
    (changed.then_some(draft), browse)
}

pub fn toggle(ui: &mut egui::Ui, label: &str, current: bool) -> Option<bool> {
    let mut value = current;
    ui.checkbox(&mut value, label).changed().then_some(value)
}

pub fn action(ui: &mut egui::Ui, label: &str, busy: bool) -> bool {
    ui.add_space(8.0);
    ui.add_enabled(!busy, egui::Button::new(label).min_size(egui::vec2(135.0, 30.0))).clicked()
}
