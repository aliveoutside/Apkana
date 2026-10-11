use eframe::egui;
use super::common;

#[derive(Debug, Clone, Default)]
pub struct BuildTabState {
    pub project_dir: String,
    pub output_apk: String,
    pub force_all: bool,
    pub strip_debug_directives: bool,
    pub run_zipalign: bool,
    pub run_sign: bool,
    pub run_install: bool,
}
impl BuildTabState {
    pub fn can_toggle_zipalign(&self) -> bool { !self.run_sign && !self.run_install }
    pub fn can_toggle_sign(&self) -> bool { !self.run_install }
}

#[derive(Debug, Clone)]
pub enum BuildMessage {
    ProjectDirChanged(String),
    OutputApkChanged(String),
    BrowseProjectDir,
    BrowseOutputApk,
    ForceAllToggled(bool),
    StripDebugDirectivesToggled(bool),
    RunZipalignToggled(bool),
    RunSignToggled(bool),
    RunInstallToggled(bool),
    Start,
}

pub fn show(ui: &mut egui::Ui, state: &BuildTabState, busy: bool) -> Vec<BuildMessage> {
    let mut events = Vec::new();
    common::card(ui, "Build APK", |ui| {
        let (changed, browse) = common::path(ui, "Decoded project directory", "/path/to/decoded-project", &state.project_dir);
        if let Some(v) = changed { events.push(BuildMessage::ProjectDirChanged(v)); }
        if browse { events.push(BuildMessage::BrowseProjectDir); }
        ui.add_space(8.0);
        let (changed, browse) = common::path(ui, "Output APK (optional)", "/path/to/output.apk", &state.output_apk);
        if let Some(v) = changed { events.push(BuildMessage::OutputApkChanged(v)); }
        if browse { events.push(BuildMessage::BrowseOutputApk); }
        ui.separator();
        ui.label(egui::RichText::new("Build options").strong());
        if let Some(v) = common::toggle(ui, "Force rebuild (-f)", state.force_all) {
            events.push(BuildMessage::ForceAllToggled(v));
        }
        if let Some(v) = common::toggle(ui, "Strip smali debug directives", state.strip_debug_directives) {
            events.push(BuildMessage::StripDebugDirectivesToggled(v));
        }
        ui.separator();
        ui.label(egui::RichText::new("Pipeline").strong());
        ui.add_enabled_ui(state.can_toggle_zipalign(), |ui| {
            if let Some(v) = common::toggle(ui, "Run zipalign", state.run_zipalign) {
                events.push(BuildMessage::RunZipalignToggled(v));
            }
        });
        ui.add_enabled_ui(state.can_toggle_sign(), |ui| {
            if let Some(v) = common::toggle(ui, "Sign APK (apksigner)", state.run_sign) {
                events.push(BuildMessage::RunSignToggled(v));
            }
        });
        if let Some(v) = common::toggle(ui, "Install via adb (-r)", state.run_install) {
            events.push(BuildMessage::RunInstallToggled(v));
        }
        if common::action(ui, "Build APK", busy) { events.push(BuildMessage::Start); }
    });
    events
}
