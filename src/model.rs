use std::fmt;

/// Severity as represented in a 3CX log record.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    Other(String),
}

impl LogLevel {
    pub fn parse(value: &str) -> Self {
        match value.trim() {
            "Trace" => Self::Trace,
            "Debug" => Self::Debug,
            "Info" => Self::Info,
            "Warn" | "Warning" => Self::Warning,
            "Error" => Self::Error,
            "Fatal" | "Critical" => Self::Fatal,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Trace => "Trace",
            Self::Debug => "Debug",
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Fatal => "Fatal",
            Self::Other(value) => value,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One logical event. Multi-line stack traces and response bodies are kept in
/// the same event, with their original line range and raw text preserved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEvent {
    pub timestamp: String,
    pub thread_id: String,
    pub level: LogLevel,
    pub source: Option<String>,
    pub message: String,
    pub raw: String,
    pub start_line: usize,
    pub end_line: usize,
    pub flow_id: Option<String>,
    pub call_id: Option<String>,
    pub component: Option<String>,
}
