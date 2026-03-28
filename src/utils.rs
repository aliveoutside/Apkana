use std::env;
use std::path::{Path, PathBuf};

use crate::config::ToolPaths;

pub fn pick_file() -> Option<String> {
    rfd::FileDialog::new().pick_file().map(path_to_string)
}

pub fn pick_folder() -> Option<String> {
    rfd::FileDialog::new().pick_folder().map(path_to_string)
}

pub fn save_file() -> Option<String> {
    rfd::FileDialog::new().save_file().map(path_to_string)
}

pub fn pick_split_archive_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Split APK archives", &["apks", "xapk", "apkm"])
        .pick_file()
        .map(path_to_string)
}

pub fn pick_apk_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("APK file", &["apk"])
        .pick_file()
        .map(path_to_string)
}

pub fn save_apk_file_with_default(default_path: &str) -> Option<String> {
    let mut dialog = rfd::FileDialog::new().add_filter("APK file", &["apk"]);
    let path = Path::new(default_path);

    if let Some(parent) = path.parent() {
        dialog = dialog.set_directory(parent);
    }
    if let Some(name) = path.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy().as_ref());
    }

    dialog.save_file().map(path_to_string)
}

pub fn discover_tools(current: &ToolPaths) -> ToolPaths {
    ToolPaths {
        java: fallback_executable(&current.java, "java"),
        apktool_jar: fallback_apktool_jar(&current.apktool_jar),
        apksigner: fallback_executable(&current.apksigner, "apksigner"),
        zipalign: fallback_executable(&current.zipalign, "zipalign"),
        adb: fallback_executable(&current.adb, "adb"),
    }
}

fn fallback_executable(current: &str, tool: &str) -> String {
    if !current.trim().is_empty() {
        return current.to_string();
    }

    if let Some(path) = which_like(tool) {
        return path_to_string(path);
    }

    tool.to_string()
}

fn fallback_apktool_jar(current: &str) -> String {
    if !current.trim().is_empty() {
        return current.to_string();
    }

    if let Some(v) = env::var_os("APKTOOL_JAR") {
        let path = PathBuf::from(v);
        if path.exists() {
            return path_to_string(path);
        }
    }

    if let Some(path) = which_like("apktool") {
        return path_to_string(path);
    }

    String::new()
}

fn which_like(tool: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for dir in env::split_paths(&paths) {
        let full = dir.join(tool);
        if full.exists() {
            return Some(full);
        }
    }
    None
}

fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().to_string()
}
