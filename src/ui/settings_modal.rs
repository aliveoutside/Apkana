use eframe::egui;
use crate::config::ToolPaths;
use super::common;
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    JavaChanged(String),
    ApktoolJarChanged(String),
    ApksignerChanged(String),
    ZipalignChanged(String),
    AdbChanged(String),
    BrowseApktoolJar,
    Save,
    Cancel,
}
pub fn show(ui: &mut egui::Ui, paths: &ToolPaths) -> Vec<SettingsMessage> {
    let mut events = Vec::new();
    ui.set_min_width(460.0);
    if let Some(v) = common::field(ui, "Java executable", "java", &paths.java) {
        events.push(SettingsMessage::JavaChanged(v));
    }
    ui.add_space(6.0);
    let (changed, browse) = common::path(ui, "apktool path (jar or executable)", "/path/to/apktool.jar", &paths.apktool_jar);
    if let Some(v) = changed { events.push(SettingsMessage::ApktoolJarChanged(v)); }
    if browse { events.push(SettingsMessage::BrowseApktoolJar); }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "apksigner executable", "apksigner", &paths.apksigner) {
        events.push(SettingsMessage::ApksignerChanged(v));
    }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "zipalign executable", "zipalign", &paths.zipalign) {
        events.push(SettingsMessage::ZipalignChanged(v));
    }
    ui.add_space(6.0);
    if let Some(v) = common::field(ui, "adb executable", "adb", &paths.adb) {
        events.push(SettingsMessage::AdbChanged(v));
    }
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() { events.push(SettingsMessage::Cancel); }
        if ui.button("Save").clicked() { events.push(SettingsMessage::Save); }
    });
    events
}
