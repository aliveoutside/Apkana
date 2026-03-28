#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod config;
mod tools;
mod ui;
mod utils;

use app::ApkanaApp;

fn main() -> iced::Result {
    iced::application(ApkanaApp::new, ApkanaApp::update, ApkanaApp::view)
        .title(ApkanaApp::title)
        .theme(ApkanaApp::theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1200.0, 900.0),
            min_size: Some(iced::Size::new(960.0, 680.0)),
            ..Default::default()
        })
        .run()
}
