use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPaths {
    pub java: String,
    pub apktool_jar: String,
    pub apksigner: String,
    pub zipalign: String,
    #[serde(default = "default_adb_executable")]
    pub adb: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RecentPaths {
    pub decode_apk_path: String,
    pub decode_output_dir: String,
    pub build_project_dir: String,
    pub build_output_apk: String,
    pub merge_input_path: String,
    pub merge_output_path: String,
    pub install_apk_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme_mode")]
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub tools: ToolPaths,
    #[serde(default)]
    pub recent: RecentPaths,
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::Dark
}

fn default_adb_executable() -> String {
    String::from("adb")
}

impl Default for ToolPaths {
    fn default() -> Self {
        Self {
            java: String::from("java"),
            apktool_jar: String::new(),
            apksigner: String::from("apksigner"),
            zipalign: String::from("zipalign"),
            adb: default_adb_executable(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark,
            tools: ToolPaths::default(),
            recent: RecentPaths::default(),
        }
    }
}

pub fn config_path() -> PathBuf {
    let portable = portable_config_path();
    if portable.exists() {
        portable
    } else {
        xdg_config_path()
    }
}

pub fn portable_config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));

    exe_dir.join("config.toml")
}

pub fn xdg_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("apkana").join("config.toml")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    let content = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return AppConfig::default(),
    };

    toml::from_str(&content).unwrap_or_default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create config dir failed: {e}"))?;
    }

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("toml serialize failed: {e}"))?;
    fs::write(path, content).map_err(|e| format!("write config failed: {e}"))
}
