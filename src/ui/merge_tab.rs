use super::common;
use eframe::egui;
#[derive(Debug, Clone, Default)]
pub struct MergeTabState {
    pub input_path: String,
    pub output_path: String,
}
#[derive(Debug, Clone)]
pub enum MergeMessage {
    InputPathChanged(String),
    OutputPathChanged(String),
    BrowseInput,
    BrowseOutput,
    Start,
}
pub fn show(ui: &mut egui::Ui, state: &MergeTabState, busy: bool) -> Vec<MergeMessage> {
    let mut events = Vec::new();
    common::card(ui, "Merge Split APKs", |ui| {
        let (changed, browse) = common::path(
            ui,
            "Input archive (.apks, .xapk, .apkm)",
            "/path/to/archive.apks",
            &state.input_path,
        );
        if let Some(v) = changed {
            events.push(MergeMessage::InputPathChanged(v));
        }
        if browse {
            events.push(MergeMessage::BrowseInput);
        }
        ui.add_space(8.0);
        let (changed, browse) =
            common::path(ui, "Output APK", "/path/to/output.apk", &state.output_path);
        if let Some(v) = changed {
            events.push(MergeMessage::OutputPathChanged(v));
        }
        if browse {
            events.push(MergeMessage::BrowseOutput);
        }
        if common::action(ui, "Merge APK", busy) {
            events.push(MergeMessage::Start);
        }
    });
    events
}
