use colored::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogMessage(Vec<String>);

impl std::ops::Deref for LogMessage {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LogMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.join(", "))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CallStackLine(String);

impl std::ops::Deref for CallStackLine {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CallStackLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for CallStackLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dimmed())
    }
}

impl CallStackLine {
    pub fn replace(&self, from: &str, to: &str) -> Self {
        CallStackLine(self.0.replace(from, to))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CallStack(pub Vec<CallStackLine>);

impl From<Option<&str>> for CallStack {
    fn from(value: Option<&str>) -> Self {
        let lines = value
            .unwrap_or("")
            .split("\n")
            .map(|line| CallStackLine(line.to_string()))
            .collect();
        Self(lines)
    }
}

impl CallStack {
    pub fn new(value: Option<&str>) -> Self {
        CallStack::from(value)
    }

    pub fn location(&self) -> CallStackLine {
        let filtered = self
            .0
            .iter()
            .filter(|line| !line.contains("node_modules") && line.contains("src"))
            .map(|line| line.replace("@http://localhost:1420/", ""))
            .collect::<Vec<CallStackLine>>()
            .clone();
        if filtered.len() > 0 {
            filtered[filtered.len() - 1].clone()
        } else {
            CallStackLine("".to_string())
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RecordPayload {
    pub message: String,
    pub level: LogLevel,
}

/// An enum representing the available verbosity levels of the logger.
///
/// It is very similar to the [`log::Level`], but serializes to unsigned ints instead of strings.
#[derive(Debug, Clone, Deserialize_repr, Serialize_repr, Default)]
#[repr(u16)]
pub enum LogLevel {
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace = 1,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "info" level.
    ///
    /// Designates useful information.
    #[default]
    Info,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(log_level: tracing::Level) -> Self {
        match log_level {
            tracing::Level::TRACE => LogLevel::Trace,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::ERROR => LogLevel::Error,
        }
    }
}
