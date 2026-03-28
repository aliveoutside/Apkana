use crate::tools::{OutputStream, ToolResult};
use crate::ui::log_panel::LogEntry;

use super::{ApkanaApp, LogLevel};

impl ApkanaApp {
    pub(super) fn push_result(&mut self, label: &str, result: Result<ToolResult, String>) {
        match result {
            Ok(result) => {
                for line in result.output {
                    let level = match line.stream {
                        OutputStream::Stdout => LogLevel::Info,
                        OutputStream::Stderr => LogLevel::Warn,
                    };
                    self.logs.push(LogEntry {
                        level,
                        line: line.line,
                    });
                }

                if result.exit_code == 0 {
                    self.status_message =
                        format!("{label} finished in {:.2}s", result.duration.as_secs_f32());
                    self.push_info(format!(
                        "{label} finished in {:.2}s",
                        result.duration.as_secs_f32()
                    ));
                } else {
                    self.status_message =
                        format!("{label} failed with exit code {}", result.exit_code);
                    self.push_error(format!(
                        "{label} failed with exit code {}",
                        result.exit_code
                    ));
                }
            }
            Err(e) => {
                self.status_message = format!("{label} error: {e}");
                self.push_error(format!("{label} error: {e}"));
            }
        }
    }

    pub(super) fn push_info(&mut self, line: impl Into<String>) {
        self.logs.push(LogEntry {
            level: LogLevel::Info,
            line: line.into(),
        });
    }

    pub(super) fn push_error(&mut self, line: impl Into<String>) {
        self.logs.push(LogEntry {
            level: LogLevel::Error,
            line: line.into(),
        });
    }

    pub(super) fn logs_for_clipboard(&self) -> String {
        self.logs
            .iter()
            .map(|entry| {
                let prefix = match entry.level {
                    LogLevel::Info => "[I]",
                    LogLevel::Warn => "[W]",
                    LogLevel::Error => "[E]",
                };

                format!("{prefix} {}", entry.line)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
