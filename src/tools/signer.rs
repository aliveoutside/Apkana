use std::process::Command;

use crate::config::ToolPaths;
use crate::tools::{ToolResult, run_command_collect};

#[derive(Debug, Clone)]
pub struct SignOptions {
    pub apk_path: String,
    pub keystore_path: String,
    pub alias: String,
    pub keystore_pass: String,
    pub key_pass: String,
}

pub fn sign(paths: &ToolPaths, options: &SignOptions) -> Result<ToolResult, String> {
    if options.apk_path.is_empty() || options.keystore_path.is_empty() || options.alias.is_empty() {
        return Err(String::from("APK, keystore, and alias are required"));
    }

    let mut command = Command::new(&paths.apksigner);
    command
        .arg("sign")
        .arg("--ks")
        .arg(&options.keystore_path)
        .arg("--ks-key-alias")
        .arg(&options.alias)
        .arg("--in")
        .arg(&options.apk_path);

    if !options.keystore_pass.is_empty() {
        command
            .arg("--ks-pass")
            .arg(format!("pass:{}", options.keystore_pass));
    }

    if !options.key_pass.is_empty() {
        command
            .arg("--key-pass")
            .arg(format!("pass:{}", options.key_pass));
    }

    run_command_collect(command)
}
