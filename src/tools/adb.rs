use std::process::Command;

use crate::config::ToolPaths;
use crate::tools::{ToolResult, run_command_collect};

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub apk_path: String,
    pub replace_existing: bool,
}

pub fn install(paths: &ToolPaths, options: &InstallOptions) -> Result<ToolResult, String> {
    if options.apk_path.trim().is_empty() {
        return Err(String::from("ADB install: APK path is required"));
    }

    let mut command = Command::new(&paths.adb);
    command.arg("install");
    if options.replace_existing {
        command.arg("-r");
    }
    command.arg(&options.apk_path);

    run_command_collect(command)
}
