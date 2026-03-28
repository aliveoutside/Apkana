use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::config::ToolPaths;
use crate::tools::{OutputStream, ToolOutput, ToolResult, run_command_collect};

const DEFAULT_JAVA_HEAP_OPTS: &str = "-Xmx4096m -XX:+UseG1GC";
const DEFAULT_APKTOOL_JOBS: &str = "2";
const RETRY_JAVA_HEAP_OPTS: &str = "-Xmx8192m -XX:+UseG1GC";
const FINAL_JAVA_HEAP_OPTS: &str = "-Xmx12288m -XX:+UseG1GC";

#[derive(Debug, Clone)]
pub struct DecodeOptions {
    pub apk_path: String,
    pub output_dir: String,
    pub force: bool,
    pub no_src: bool,
    pub no_res: bool,
}

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub project_dir: String,
    pub output_apk: String,
    pub force_all: bool,
    pub strip_debug_directives: bool,
}

pub fn decode(paths: &ToolPaths, options: &DecodeOptions) -> Result<ToolResult, String> {
    if options.apk_path.is_empty() || options.output_dir.is_empty() {
        return Err(String::from("APK path and output dir are required"));
    }

    let mut command = build_apktool_command(paths, DEFAULT_JAVA_HEAP_OPTS)?;
    command
        .arg("d")
        .arg(&options.apk_path)
        .arg("-o")
        .arg(&options.output_dir);

    if options.force {
        command.arg("-f");
    }
    if options.no_src {
        command.arg("-s");
    }
    if options.no_res {
        command.arg("-r");
    }

    run_command_collect(command)
}

pub fn build(paths: &ToolPaths, options: &BuildOptions) -> Result<ToolResult, String> {
    if options.project_dir.is_empty() {
        return Err(String::from("project directory is required"));
    }

    let attempts = [
        (DEFAULT_JAVA_HEAP_OPTS, DEFAULT_APKTOOL_JOBS),
        (RETRY_JAVA_HEAP_OPTS, DEFAULT_APKTOOL_JOBS),
        (FINAL_JAVA_HEAP_OPTS, "1"),
    ];

    let mut all_output = Vec::new();
    let mut total_duration = Duration::default();
    let mut last_exit_code = -1;

    if options.strip_debug_directives {
        let (changed_files, removed_lines) = strip_smali_debug_directives(&options.project_dir)?;
        all_output.push(ToolOutput {
            stream: OutputStream::Stdout,
            line: format!(
                "OOM workaround enabled: stripped {removed_lines} debug directive lines in {changed_files} smali files"
            ),
        });
    }

    for (index, (heap_opts, jobs)) in attempts.into_iter().enumerate() {
        if index > 0 {
            all_output.push(ToolOutput {
                stream: OutputStream::Stderr,
                line: format!("OOM detected; retrying build with {heap_opts} and -j {jobs}"),
            });
        }

        let mut command = build_apktool_command(paths, heap_opts)?;
        command
            .arg("b")
            .arg(&options.project_dir)
            .arg("-j")
            .arg(jobs);

        if !options.output_apk.is_empty() {
            command.arg("-o").arg(&options.output_apk);
        }
        if options.force_all {
            command.arg("-f");
        }
        let result = run_command_collect(command)?;
        total_duration += result.duration;
        last_exit_code = result.exit_code;
        let attempt_output = result.output;
        let attempt_has_oom = contains_out_of_memory(&attempt_output);
        all_output.extend(attempt_output);

        if last_exit_code == 0 {
            return Ok(ToolResult {
                exit_code: 0,
                duration: total_duration,
                output: all_output,
            });
        }

        if !attempt_has_oom {
            return Ok(ToolResult {
                exit_code: last_exit_code,
                duration: total_duration,
                output: all_output,
            });
        }
    }

    Ok(ToolResult {
        exit_code: last_exit_code,
        duration: total_duration,
        output: all_output,
    })
}

fn build_apktool_command(paths: &ToolPaths, heap_opts: &str) -> Result<Command, String> {
    let value = paths.apktool_jar.trim();
    if value.is_empty() {
        return Err(String::from(
            "apktool path is not configured (set jar or executable)",
        ));
    }

    if value.ends_with(".jar") {
        let mut command = Command::new(&paths.java);
        command.arg("-jar").arg(value);
        apply_java_heap_env(&mut command, heap_opts);
        return Ok(command);
    }

    let mut command = Command::new(value);
    apply_java_heap_env(&mut command, heap_opts);
    Ok(command)
}

fn apply_java_heap_env(command: &mut Command, heap_opts: &str) {
    let merged = match std::env::var("JAVA_TOOL_OPTIONS") {
        Ok(existing) => {
            let cleaned = remove_heap_opts(&existing);
            if cleaned.is_empty() {
                String::from(heap_opts)
            } else {
                format!("{cleaned} {heap_opts}")
            }
        }
        Err(_) => String::from(heap_opts),
    };

    command.env("JAVA_TOOL_OPTIONS", merged);
}

fn remove_heap_opts(options: &str) -> String {
    options
        .split_whitespace()
        .filter(|opt| !opt.starts_with("-Xmx") && !opt.starts_with("-Xms"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_out_of_memory(output: &[ToolOutput]) -> bool {
    output
        .iter()
        .any(|line| line.line.to_ascii_lowercase().contains("outofmemoryerror"))
}

fn strip_smali_debug_directives(project_dir: &str) -> Result<(usize, usize), String> {
    let mut stack = vec![PathBuf::from(project_dir)];
    let mut changed_files = 0usize;
    let mut removed_lines = 0usize;

    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("failed to read directory '{}': {e}", dir.display()))?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                format!("failed to read entry in '{}': {e}", dir.as_path().display())
            })?;
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if !is_smali_file(&path) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue,
            };

            let mut changed = false;
            let mut next = String::with_capacity(content.len());

            for line in content.lines() {
                if is_debug_directive(line) {
                    removed_lines += 1;
                    changed = true;
                } else {
                    next.push_str(line);
                    next.push('\n');
                }
            }

            if changed {
                fs::write(&path, next).map_err(|e| {
                    format!("failed to rewrite smali file '{}': {e}", path.display())
                })?;
                changed_files += 1;
            }
        }
    }

    Ok((changed_files, removed_lines))
}

fn is_smali_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("smali")
}

fn is_debug_directive(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(".line ")
        || trimmed == ".line"
        || trimmed.starts_with(".local ")
        || trimmed == ".local"
        || trimmed.starts_with(".end local")
        || trimmed.starts_with(".restart local")
        || trimmed.starts_with(".prologue")
        || trimmed.starts_with(".epilogue")
}
