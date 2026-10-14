use crate::app::LogLevel;
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub line: String,
}
