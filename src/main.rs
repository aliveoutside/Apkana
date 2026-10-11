#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod config;
mod tools;
mod ui;
mod utils;

use app::ApkanaApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Apkana",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 900.0])
                .with_min_inner_size([960.0, 680.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let app = ApkanaApp::default();
            app.apply_theme(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
