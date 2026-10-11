use eframe::egui;
use super::common;

#[derive(Debug, Clone, Default)]
pub struct DecodeTabState {
    pub apk_path: String,
    pub output_dir: String,
    pub force: bool,
    pub no_src: bool,
    pub no_res: bool,
}

#[derive(Debug, Clone)]
pub enum DecodeMessage {
    ApkPathChanged(String),
    OutputDirChanged(String),
    BrowseApk,
    BrowseOutputDir,
    ForceToggled(bool),
    NoSrcToggled(bool),
    NoResToggled(bool),
    Start,
}

pub fn show(ui: &mut egui::Ui, state: &DecodeTabState, busy: bool) -> Vec<DecodeMessage> {
    let mut events = Vec::new();
    common::card(ui, "Decode APK", |ui| {
        let (changed, browse) = common::path(ui, "APK file", "/path/to/app.apk", &state.apk_path);
        {
            if let Some(v) = changed { events.push(DecodeMessage::ApkPathChanged(v)); }
            if browse { events.push(DecodeMessage::BrowseApk); }
        }
        ui.add_space(8.0);
        let (changed, browse) = common::path(ui, "Output directory", "/path/to/output", &state.output_dir);
        if let Some(v) = changed { events.push(DecodeMessage::OutputDirChanged(v)); }
        if browse { events.push(DecodeMessage::BrowseOutputDir); }
        ui.separator();
        ui.label(egui::RichText::new("Options").strong());
        if let Some(v) = common::toggle(ui, "Force overwrite existing output (-f)", state.force) {
            events.push(DecodeMessage::ForceToggled(v));
        }
        if let Some(v) = common::toggle(ui, "Skip sources (-s)", state.no_src) {
            events.push(DecodeMessage::NoSrcToggled(v));
        }
        if let Some(v) = common::toggle(ui, "Skip resources (-r)", state.no_res) {
            events.push(DecodeMessage::NoResToggled(v));
        }
        if common::action(ui, "Decode APK", busy) { events.push(DecodeMessage::Start); }
    });
    events
}
