use super::common;
use eframe::egui;
#[derive(Debug, Clone, Default)]
pub struct InstallTabState {
    pub apk_path: String,
}
#[derive(Debug, Clone)]
pub enum InstallMessage {
    ApkPathChanged(String),
    BrowseApk,
    Start,
}
pub fn show(ui: &mut egui::Ui, state: &InstallTabState, busy: bool) -> Vec<InstallMessage> {
    let mut events = Vec::new();
    common::card(ui, "Install APK", |ui| {
        let (changed, browse) = common::path(ui, "APK file", "/path/to/app.apk", &state.apk_path);
        if let Some(v) = changed {
            events.push(InstallMessage::ApkPathChanged(v));
        }
        if browse {
            events.push(InstallMessage::BrowseApk);
        }
        common::hint(ui, "Requires a connected Android device with USB debugging enabled.");
        if common::action(ui, "Install APK", busy) {
            events.push(InstallMessage::Start);
        }
    });
    events
}
