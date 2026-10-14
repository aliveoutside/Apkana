use super::common;
use crate::config::ToolPaths;
use eframe::egui;
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    JavaChanged(String),
    ApktoolJarChanged(String),
    ApksignerChanged(String),
    ZipalignChanged(String),
    Aapt2Changed(String),
    AdbChanged(String),
    BrowseApktoolJar,
    Save,
    Cancel,
}
pub fn show(ui: &mut egui::Ui, paths: &ToolPaths) -> Vec<SettingsMessage> {
    let mut events = Vec::new();
    ui.set_min_width(480.0);
    let p = common::palette(ui.visuals().dark_mode);
    ui.label(
        egui::RichText::new("Toolchain")
            .size(20.0)
            .strong()
            .color(p.text),
    );
    ui.label(
        egui::RichText::new("Configure the executables used by Android operations.")
            .size(12.0)
            .color(p.muted),
    );
    ui.add_space(17.0);
    if let Some(v) = common::field(ui, "Java executable", "java", &paths.java) {
        events.push(SettingsMessage::JavaChanged(v));
    }
    ui.add_space(6.0);
    let (changed, browse) = common::path(
        ui,
        "apktool path (jar or executable)",
        "/path/to/apktool.jar",
        &paths.apktool_jar,
    );
    if let Some(v) = changed {
        events.push(SettingsMessage::ApktoolJarChanged(v));
    }
    if browse {
        events.push(SettingsMessage::BrowseApktoolJar);
    }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "apksigner executable", "apksigner", &paths.apksigner) {
        events.push(SettingsMessage::ApksignerChanged(v));
    }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "zipalign executable", "zipalign", &paths.zipalign) {
        events.push(SettingsMessage::ZipalignChanged(v));
    }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "aapt2 executable", "aapt2", &paths.aapt2) {
        events.push(SettingsMessage::Aapt2Changed(v));
    }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "adb executable", "adb", &paths.adb) {
        events.push(SettingsMessage::AdbChanged(v));
    }
    ui.add_space(19.0);
    ui.separator();
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui.add_sized([100.0, 35.0], egui::Button::new("Cancel")).clicked() {
            events.push(SettingsMessage::Cancel);
        }
        if ui
            .add_sized(
                [130.0, 35.0],
                egui::Button::new(
                    egui::RichText::new("Save changes")
                        .strong()
                        .color(p.accent_text),
                )
                .fill(p.accent)
                .stroke(egui::Stroke::NONE)
                .corner_radius(egui::CornerRadius::same(9)),
            )
            .clicked()
        {
            events.push(SettingsMessage::Save);
        }
    });
    events
}
