//! Shared visual language and form controls for the desktop interface.
use eframe::egui;

#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas: egui::Color32,
    pub sidebar: egui::Color32,
    pub surface: egui::Color32,
    pub surface_hover: egui::Color32,
    pub border: egui::Color32,
    pub accent: egui::Color32,
    pub accent_text: egui::Color32,
    pub text: egui::Color32,
    pub muted: egui::Color32,
    pub field: egui::Color32,
    pub selected: egui::Color32,
}

pub fn palette(dark: bool) -> Palette {
    use egui::Color32 as C;
    if dark {
        Palette {
            canvas: C::from_rgb(15, 18, 20),
            sidebar: C::from_rgb(20, 23, 26),
            surface: C::from_rgb(26, 30, 33),
            surface_hover: C::from_rgb(35, 41, 43),
            border: C::from_rgb(47, 53, 56),
            accent: C::from_rgb(156, 225, 139),
            accent_text: C::from_rgb(17, 32, 19),
            text: C::from_rgb(240, 244, 240),
            muted: C::from_rgb(151, 160, 158),
            field: C::from_rgb(21, 25, 28),
            selected: C::from_rgb(42, 60, 46),
        }
    } else {
        Palette {
            canvas: C::from_rgb(246, 248, 245),
            sidebar: C::from_rgb(236, 241, 235),
            surface: C::WHITE,
            surface_hover: C::from_rgb(226, 235, 225),
            border: C::from_rgb(218, 225, 217),
            accent: C::from_rgb(48, 118, 67),
            accent_text: C::WHITE,
            text: C::from_rgb(30, 43, 36),
            muted: C::from_rgb(101, 118, 108),
            field: C::from_rgb(247, 250, 247),
            selected: C::from_rgb(209, 231, 207),
        }
    }
}

pub fn card(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    let p = palette(ui.visuals().dark_mode);
    let (eyebrow, description) = match title {
        "Decode APK" => ("01 / DECODE", "Unpack an APK into an editable project."),
        "Build APK" => ("02 / BUILD", "Compile your project and prepare a release."),
        "Sign APK" => ("03 / SIGN", "Apply a signing key to an Android package."),
        "Merge Split APKs" => ("04 / MERGE", "Combine compatible split packages into one APK."),
        "Install APK" => ("05 / INSTALL", "Deploy an APK to a connected Android device."),
        _ => ("APKANA / TOOL", ""),
    };

    egui::Frame::new()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(15))
        .inner_margin(egui::Margin::same(22))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(0.0));
            ui.label(
                egui::RichText::new(eyebrow)
                    .size(10.0)
                    .strong()
                    .color(p.accent),
            );
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new(title)
                    .size(20.0)
                    .strong()
                    .color(p.text),
            );
            ui.add_space(3.0);
            ui.label(egui::RichText::new(description).size(12.5).color(p.muted));
            ui.add_space(20.0);
            content(ui);
        });
}

pub fn section(ui: &mut egui::Ui, name: &str) {
    let p = palette(ui.visuals().dark_mode);
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(9.0);
    ui.label(egui::RichText::new(name).size(11.0).strong().color(p.muted));
    ui.add_space(3.0);
}

pub fn hint(ui: &mut egui::Ui, text: &str) {
    let p = palette(ui.visuals().dark_mode);
    ui.add_space(8.0);
    ui.label(egui::RichText::new(text).size(12.0).color(p.muted));
}

pub fn field(ui: &mut egui::Ui, label: &str, hint: &str, value: &str) -> Option<String> {
    edit(ui, label, hint, value, false)
}

pub fn secret(ui: &mut egui::Ui, label: &str, hint: &str, value: &str) -> Option<String> {
    edit(ui, label, hint, value, true)
}

fn edit(ui: &mut egui::Ui, label: &str, hint: &str, value: &str, secret: bool) -> Option<String> {
    field_label(ui, label);
    let mut draft = value.to_owned();
    let response = ui.add(
        egui::TextEdit::singleline(&mut draft)
            .hint_text(hint)
            .password(secret)
            .desired_width(f32::INFINITY),
    );
    response.changed().then_some(draft)
}

fn field_label(ui: &mut egui::Ui, label: &str) {
    let p = palette(ui.visuals().dark_mode);
    ui.label(egui::RichText::new(label).size(12.5).color(p.muted));
    ui.add_space(5.0);
}

pub fn path(ui: &mut egui::Ui, label: &str, hint: &str, value: &str) -> (Option<String>, bool) {
    field_label(ui, label);
    let mut draft = value.to_owned();
    let mut changed = false;
    let mut browse = false;
    ui.horizontal(|ui| {
        changed = ui
            .add(
                egui::TextEdit::singleline(&mut draft)
                    .hint_text(hint)
                    .desired_width((ui.available_width() - 102.0).max(88.0)),
            )
            .changed();
        browse = ui
            .add_sized([93.0, 31.0], egui::Button::new("Browse..."))
            .clicked();
    });
    (changed.then_some(draft), browse)
}

pub fn toggle(ui: &mut egui::Ui, label: &str, current: bool) -> Option<bool> {
    let mut value = current;
    ui.add_space(3.0);
    ui.add(egui::Checkbox::new(
        &mut value,
        egui::RichText::new(label).size(13.0),
    ))
    .changed()
    .then_some(value)
}

pub fn action(ui: &mut egui::Ui, label: &str, busy: bool) -> bool {
    let p = palette(ui.visuals().dark_mode);
    ui.add_space(19.0);
    ui.add_enabled(
        !busy,
        egui::Button::new(
            egui::RichText::new(if busy { "Working..." } else { label })
                .strong()
                .color(p.accent_text),
        )
        .fill(p.accent)
        .stroke(egui::Stroke::NONE)
        .corner_radius(egui::CornerRadius::same(9))
        .min_size(egui::vec2(155.0, 39.0)),
    )
    .clicked()
}
