use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::config::{self, AppConfig};
use crate::tools::ToolResult;
use crate::ui::build_tab::{BuildMessage, BuildTabState};
use crate::ui::decode_tab::{DecodeMessage, DecodeTabState};
use crate::ui::install_tab::{InstallMessage, InstallTabState};
use crate::ui::log_panel::LogEntry;
use crate::ui::merge_tab::{MergeMessage, MergeTabState};
use crate::ui::settings_modal::SettingsMessage;
use crate::ui::sign_tab::{SignMessage, SignTabState};
use crate::ui::tabs::MainTab;
use crate::utils;

mod handlers;
mod logging;
mod persistence;
mod pipeline;
mod validation;
mod view;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(MainTab),
    Decode(DecodeMessage),
    Build(BuildMessage),
    Sign(SignMessage),
    Merge(MergeMessage),
    Install(InstallMessage),
    SettingsPressed,
    Settings(SettingsMessage),
    DecodeCompleted(Result<ToolResult, String>),
    BuildCompleted(Result<ToolResult, String>),
    ZipalignCompleted(Result<ToolResult, String>),
    SignCompleted(Result<ToolResult, String>),
    InstallCompleted(Result<ToolResult, String>),
    MergeCompleted(Result<ToolResult, String>),
    ClearLogs,
}

/// One background operation; the GUI thread owns all state transitions.
pub struct Task<T> {
    start: Option<Box<dyn FnOnce(Sender<T>) + Send>>,
}

impl<T: Send + 'static> Task<T> {
    pub fn none() -> Self {
        Self { start: None }
    }

    pub fn perform<R, F, M>(work: F, finish: M) -> Self
    where
        R: Send + 'static,
        F: FnOnce() -> R + Send + 'static,
        M: FnOnce(R) -> T + Send + 'static,
    {
        Self {
            start: Some(Box::new(move |sender| {
                thread::spawn(move || {
                    let _ = sender.send(finish(work()));
                });
            })),
        }
    }

    fn spawn(self, sender: Sender<T>) {
        if let Some(start) = self.start {
            start(sender);
        }
    }
}

pub struct ApkanaApp {
    config: AppConfig,
    settings_draft: Option<config::ToolPaths>,
    active_tab: MainTab,
    decode: DecodeTabState,
    build: BuildTabState,
    sign: SignTabState,
    merge: MergeTabState,
    install: InstallTabState,
    logs: Vec<LogEntry>,
    show_settings: bool,
    busy: bool,
    pending_sign_after_build: bool,
    pending_sign_apk_path: Option<String>,
    pending_install_after_build: bool,
    pending_install_after_sign: bool,
    pending_install_apk_path: Option<String>,
    status_message: String,
    progress_value: f32,
    events_tx: Sender<Message>,
    events_rx: Receiver<Message>,
}

impl Default for ApkanaApp {
    fn default() -> Self {
        let mut config = config::load_config();
        config.tools = utils::discover_tools(&config.tools);

        let decode = DecodeTabState {
            apk_path: config.recent.decode_apk_path.clone(),
            output_dir: config.recent.decode_output_dir.clone(),
            ..DecodeTabState::default()
        };
        let build = BuildTabState {
            project_dir: config.recent.build_project_dir.clone(),
            output_apk: config.recent.build_output_apk.clone(),
            ..BuildTabState::default()
        };
        let merge = MergeTabState {
            input_path: config.recent.merge_input_path.clone(),
            output_path: config.recent.merge_output_path.clone(),
        };
        let install = InstallTabState {
            apk_path: config.recent.install_apk_path.clone(),
        };
        let (events_tx, events_rx) = mpsc::channel();

        Self {
            config,
            settings_draft: None,
            active_tab: MainTab::DecodeBuild,
            decode,
            build,
            sign: SignTabState::default(),
            merge,
            install,
            logs: Vec::new(),
            show_settings: false,
            busy: false,
            pending_sign_after_build: false,
            pending_sign_apk_path: None,
            pending_install_after_build: false,
            pending_install_after_sign: false,
            pending_install_apk_path: None,
            status_message: String::new(),
            progress_value: 0.0,
            events_tx,
            events_rx,
        }
    }
}

impl ApkanaApp {
    fn dispatch(&mut self, message: Message) {
        self.handle(message).spawn(self.events_tx.clone());
    }

    fn drain_events(&mut self) {
        while let Ok(message) = self.events_rx.try_recv() {
            self.dispatch(message);
        }
    }

    fn handle(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            Message::SettingsPressed => {
                self.settings_draft = Some(self.config.tools.clone());
                self.show_settings = true;
                Task::none()
            }
            Message::Settings(msg) => self.update_settings(msg),
            Message::Decode(msg) => self.update_decode(msg),
            Message::Build(msg) => self.update_build(msg),
            Message::Sign(msg) => self.update_sign(msg),
            Message::Merge(msg) => self.update_merge(msg),
            Message::Install(msg) => self.update_install(msg),
            Message::DecodeCompleted(result) => {
                self.busy = false;
                self.progress_value = 0.0;
                self.push_result("Decode", result);
                Task::none()
            }
            Message::BuildCompleted(result) => self.handle_build_completed(result),
            Message::ZipalignCompleted(result) => self.handle_zipalign_completed(result),
            Message::SignCompleted(result) => self.handle_sign_completed(result),
            Message::InstallCompleted(result) => self.handle_install_completed(result),
            Message::MergeCompleted(result) => self.handle_merge_completed(result),
            Message::ClearLogs => {
                self.logs.clear();
                Task::none()
            }
        }
    }
}

impl eframe::App for ApkanaApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();
        self.render_frame(ctx);
        if self.busy {
            ctx.request_repaint_after(Duration::from_millis(80));
        }
    }
}
