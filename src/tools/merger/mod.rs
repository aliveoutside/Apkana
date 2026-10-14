mod apk_merge;
mod archive;
mod dex;
mod manifest;
mod metadata;
mod resources;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::config::ToolPaths;
use crate::tools::signer::SignOptions;
use crate::tools::{OutputStream, ToolOutput, ToolResult, run_command_collect, signer, zipalign};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MergeOptions {
    pub input_path: String,
    pub output_path: String,
}

pub fn merge(
    paths: &ToolPaths,
    options: MergeOptions,
    mut signing: SignOptions,
) -> Result<ToolResult, String> {
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

    if signing.keystore_path.trim().is_empty()
        || signing.alias.trim().is_empty()
        || !Path::new(&signing.keystore_path).is_file()
    {
        return Err("Merge: signing requires a keystore and alias; configure the Sign tab or create ~/.android/debug.keystore".into());
    }
    let stage =
        tempfile::tempdir().map_err(|e| format!("Merge: cannot create staging directory: {e}"))?;
    // A signed, compatible input set is required. Never fuse APKs from different publishers.
    let mut certificate = None;
    for (i, data) in std::iter::once(&split_set.base_apk)
        .chain(split_set.splits.iter().map(|s| &s.data))
        .enumerate()
    {
        let path = stage.path().join(format!("input-{i}.apk"));
        fs::write(&path, data).map_err(|e| format!("Merge: cannot stage input: {e}"))?;
        let mut command = Command::new(&paths.apksigner);
        command.args(["verify", "--print-certs"]).arg(&path);
        let result = checked_command(command, "input signature verification", &mut output)?;
        let certs: Vec<_> = result
            .output
            .iter()
            .filter(|o| o.line.contains("certificate SHA-256 digest:"))
            .map(|o| o.line.clone())
            .collect();
        if certs.is_empty() {
            return Err("Merge: apksigner did not report an input signing certificate".into());
        }
        if certificate.as_ref().is_some_and(|c| c != &certs) {
            return Err("Merge: split APK signing certificates differ".into());
        }
        certificate = Some(certs);
    }
    let merged_apk = apk_merge::merge_splits(&split_set.base_apk, &split_set.splits, &mut output)?;
    let unsigned = stage.path().join("unsigned.apk");
    let aligned = stage.path().join("aligned.apk");
    fs::write(&unsigned, &merged_apk)
        .map_err(|e| format!("Merge: cannot stage merged APK: {e}"))?;
    checked_result(
        zipalign::run(
            paths,
            &zipalign::ZipalignOptions {
                input_apk: unsigned.to_string_lossy().into_owned(),
                output_apk: aligned.to_string_lossy().into_owned(),
            },
        )?,
        "zipalign",
        &mut output,
    )?;
    signing.apk_path = aligned.to_string_lossy().into_owned();
    checked_result(signer::sign(paths, &signing)?, "signing", &mut output)?;
    let mut verify = Command::new(&paths.apksigner);
    verify.args(["verify", "--verbose"]).arg(&aligned);
    checked_command(verify, "output signature verification", &mut output)?;
    let mut alignment = Command::new(&paths.zipalign);
    alignment.args(["-c", "4"]).arg(&aligned);
    checked_command(alignment, "output alignment verification", &mut output)?;
    for what in ["badging", "resources"] {
        let mut command = Command::new(&paths.aapt2);
        command.args(["dump", what]).arg(&aligned);
        let result =
            run_command_collect(command).map_err(|e| format!("Merge: aapt2 {what}: {e}"))?;
        // Resource dumps can be enormous; keep successful validation logs concise.
        if result.exit_code != 0 {
            checked_result(result, &format!("aapt2 {what}"), &mut output)?;
        }
        push_info(&mut output, format!("aapt2 {what}: passed"));
    }
    let signed =
        fs::read(&aligned).map_err(|e| format!("Merge: cannot read signed output: {e}"))?;
    let metadata = metadata::inspect(&signed)?;
    if metadata.split.is_some() || metadata.split_required || !metadata.required_types.is_empty() {
        return Err("Merge: output still declares a split identity".into());
    }
    write_atomically(output_path, &signed)?;
    push_info(
        &mut output,
        format!(
            "Wrote signed and verified APK: {}",
            output_path.to_string_lossy()
        ),
    );

    Ok(ToolResult {
        exit_code: 0,
        duration: started.elapsed(),
        output,
    })
}

fn checked_result(
    result: ToolResult,
    step: &str,
    output: &mut Vec<ToolOutput>,
) -> Result<ToolResult, String> {
    if result.exit_code != 0 {
        let detail = result
            .output
            .iter()
            .map(|o| o.line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "Merge: {step} failed (exit {}): {detail}",
            result.exit_code
        ));
    }
    output.extend(result.output.clone());
    Ok(result)
}
fn checked_command(
    command: Command,
    step: &str,
    output: &mut Vec<ToolOutput>,
) -> Result<ToolResult, String> {
    checked_result(
        run_command_collect(command).map_err(|e| format!("Merge: {step}: {e}"))?,
        step,
        output,
    )
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
    let input =
        fs::canonicalize(input).map_err(|e| format!("Merge: cannot resolve input path: {e}"))?;
    let output_absolute = if output.exists() {
        fs::canonicalize(output).map_err(|e| format!("Merge: cannot resolve output path: {e}"))?
    } else {
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
        parent.join(
            output
                .file_name()
                .ok_or_else(|| String::from("Merge: invalid output filename"))?,
        )
    };
    if input == output_absolute {
        return Err(String::from(
            "Merge: input archive and output APK must be different files",
        ));
    }
    Ok(())
}

fn write_atomically(destination: &Path, data: &[u8]) -> Result<(), String> {
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|e| format!("Merge: cannot create output directory: {e}"))?;
    let name = destination
        .file_name()
        .ok_or_else(|| String::from("Merge: invalid output filename"))?;

    // Exclusive creation prevents collisions between concurrent app instances.
    let mut temp = None;
    for attempt in 0..100 {
        let candidate = parent.join(format!(
            ".{}-apkana-{}-{attempt}.tmp",
            name.to_string_lossy(),
            std::process::id()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temp = Some((candidate, file));
                break;
            }
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
        let path =
            std::env::temp_dir().join(format!("apkana-merge-alias-{}.apks", std::process::id()));
        fs::write(&path, b"source").expect("create input");
        let rejected = refuse_same_input_output(&path, &path).is_err();
        let input_kept = fs::read(&path).unwrap() == b"source";
        let _ = fs::remove_file(&path);
        assert!(rejected);
        assert!(input_kept);
    }
}
