mod apk_merge;
mod archive;
mod manifest;
mod resources;

use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::tools::{OutputStream, ToolOutput, ToolResult};

#[derive(Debug, Clone)]
pub struct MergeOptions {
    pub input_path: String,
    pub output_path: String,
}

pub fn merge(options: MergeOptions) -> Result<ToolResult, String> {
    if options.input_path.trim().is_empty() {
        return Err(String::from("Merge: input archive path is required"));
    }
    if options.output_path.trim().is_empty() {
        return Err(String::from("Merge: output APK path is required"));
    }

    let started = Instant::now();
    let mut output = Vec::new();

    let input_path = Path::new(&options.input_path);
    let output_path = Path::new(&options.output_path);

    push_info(
        &mut output,
        format!("Merge input: {}", input_path.to_string_lossy()),
    );

    let format = archive::detect_format(input_path)?;
    push_info(&mut output, format!("Detected format: {}", format.name()));

    let split_set = archive::extract_split_set(input_path, format)?;
    for warning in split_set.warnings {
        push_warn(&mut output, warning);
    }

    let merged_apk = if split_set.splits.is_empty() {
        push_info(
            &mut output,
            String::from("No split APK modules found; copying base APK as output"),
        );
        split_set.base_apk
    } else {
        push_info(
            &mut output,
            format!("Merging {} split APK modules", split_set.splits.len()),
        );
        apk_merge::merge_splits(&split_set.base_apk, &split_set.splits, &mut output)?
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Merge: failed to create output directory: {e}"))?;
    }

    fs::write(output_path, merged_apk)
        .map_err(|e| format!("Merge: failed to write output APK: {e}"))?;
    push_info(
        &mut output,
        format!("Wrote merged APK: {}", output_path.to_string_lossy()),
    );

    Ok(ToolResult {
        exit_code: 0,
        duration: started.elapsed(),
        output,
    })
}

fn push_info(output: &mut Vec<ToolOutput>, line: impl Into<String>) {
    output.push(ToolOutput {
        line: line.into(),
        stream: OutputStream::Stdout,
    });
}

fn push_warn(output: &mut Vec<ToolOutput>, line: impl Into<String>) {
    output.push(ToolOutput {
        line: line.into(),
        stream: OutputStream::Stderr,
    });
}
