mod apk_merge;
mod archive;
mod manifest;
mod resources;

use std::fs::{self, OpenOptions};
use std::io::Write;
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
    // Compare canonicalized paths to prevent replacing the source archive,
    // including when the user selects the same file through a symlink.
    refuse_same_input_output(input_path, output_path)?;

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

    let did_merge = !split_set.splits.is_empty();
    let merged_apk = if !did_merge {
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

    // Write in the target directory and then rename; a failed write cannot
    // truncate an existing output. On systems where an existing target cannot
    // be atomically replaced, fail without removing the previous APK.
    write_atomically(output_path, &merged_apk)?;
    if did_merge {
        push_warn(&mut output, "Merged APK is unsigned. Sign it with the Sign tab before installing.");
    }
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

fn refuse_same_input_output(input: &Path, output: &Path) -> Result<(), String> {
    let input = fs::canonicalize(input)
        .map_err(|e| format!("Merge: cannot resolve input path: {e}"))?;
    let output_absolute = if output.exists() {
        fs::canonicalize(output)
            .map_err(|e| format!("Merge: cannot resolve output path: {e}"))?
    } else {
        let parent = output.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
        parent.join(output.file_name().ok_or_else(|| String::from("Merge: invalid output filename"))?)
    };
    if input == output_absolute {
        return Err(String::from("Merge: input archive and output APK must be different files"));
    }
    Ok(())
}

fn write_atomically(destination: &Path, data: &[u8]) -> Result<(), String> {
    let parent = destination.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|e| format!("Merge: cannot create output directory: {e}"))?;
    let name = destination.file_name()
        .ok_or_else(|| String::from("Merge: invalid output filename"))?;

    // Exclusive creation prevents collisions between concurrent app instances.
    let mut temp = None;
    for attempt in 0..100 {
        let candidate = parent.join(format!(".{}-apkana-{}-{attempt}.tmp",
            name.to_string_lossy(), std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(file) => { temp = Some((candidate, file)); break; }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Merge: cannot create temporary output: {e}")),
        }
    }
    let (path, mut file) = temp.ok_or_else(|| String::from("Merge: no free temporary filename"))?;
    let result = (|| {
        file.write_all(data)
            .map_err(|e| format!("Merge: failed to write temporary APK: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Merge: failed to flush temporary APK: {e}"))?;
        drop(file);
        fs::rename(&path, destination)
            .map_err(|e| format!("Merge: failed to replace output APK: {e}"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&path);
    }
    result
}

#[cfg(test)]
mod output_safety_tests {
    use super::*;

    #[test]
    fn rejects_input_output_alias() {
        let path = std::env::temp_dir()
            .join(format!("apkana-merge-alias-{}.apks", std::process::id()));
        fs::write(&path, b"source").expect("create input");
        let rejected = refuse_same_input_output(&path, &path).is_err();
        let input_kept = fs::read(&path).unwrap() == b"source";
        let _ = fs::remove_file(&path);
        assert!(rejected);
        assert!(input_kept);
    }
}
