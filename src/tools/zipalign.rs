use std::process::Command;

use crate::config::ToolPaths;
use crate::tools::{ToolResult, run_command_collect};

#[derive(Debug, Clone)]
pub struct ZipalignOptions {
    pub input_apk: String,
    pub output_apk: String,
}

pub fn run(paths: &ToolPaths, options: &ZipalignOptions) -> Result<ToolResult, String> {
    if options.input_apk.is_empty() || options.output_apk.is_empty() {
        return Err(String::from("zipalign input and output are required"));
    }

    let mut command = Command::new(&paths.zipalign);
    command
        .arg("-f")
        .arg("4")
        .arg(&options.input_apk)
        .arg(&options.output_apk);

    run_command_collect(command)
}
