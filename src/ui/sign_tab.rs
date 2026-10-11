use super::common;
use eframe::egui;

#[derive(Debug, Clone, Default)]
pub struct SignTabState {
    pub apk_path: String,
    pub keystore_path: String,
    pub alias: String,
    pub keystore_pass: String,
    pub key_pass: String,
}

#[derive(Debug, Clone)]
pub enum SignMessage {
    ApkPathChanged(String),
    KeystorePathChanged(String),
    AliasChanged(String),
    KeystorePassChanged(String),
    KeyPassChanged(String),
    BrowseApk,
    BrowseKeystore,
    Start,
}

pub fn show(ui: &mut egui::Ui, state: &SignTabState, busy: bool) -> Vec<SignMessage> {
    let mut events = Vec::new();
    common::card(ui, "Sign APK", |ui| {
        let (changed, browse) = common::path(ui, "APK file", "/path/to/app.apk", &state.apk_path);
        if let Some(v) = changed {
            events.push(SignMessage::ApkPathChanged(v));
        }
        if browse {
            events.push(SignMessage::BrowseApk);
        }
        ui.add_space(8.0);
        let (changed, browse) = common::path(
            ui,
            "Keystore file (empty for debug.keystore)",
            "/path/to/keystore.jks",
            &state.keystore_path,
        );
        if let Some(v) = changed {
            events.push(SignMessage::KeystorePathChanged(v));
        }
        if browse {
            events.push(SignMessage::BrowseKeystore);
        }
        ui.add_space(8.0);
        if let Some(v) = common::field(ui, "Key alias", "Key alias (optional)", &state.alias) {
            events.push(SignMessage::AliasChanged(v));
        }
        ui.add_space(8.0);
        if let Some(v) = common::secret(
            ui,
            "Keystore password",
            "Keystore password",
            &state.keystore_pass,
        ) {
            events.push(SignMessage::KeystorePassChanged(v));
        }
        ui.add_space(8.0);
        if let Some(v) = common::secret(ui, "Key password", "Key password", &state.key_pass) {
            events.push(SignMessage::KeyPassChanged(v));
        }
        if common::action(ui, "Sign APK", busy) {
            events.push(SignMessage::Start);
        }
    });
    events
}
