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

impl From<LogLevel> for tracing_log::log::Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => tracing_log::log::Level::Trace,
            LogLevel::Debug => tracing_log::log::Level::Debug,
            LogLevel::Info => tracing_log::log::Level::Info,
            LogLevel::Warn => tracing_log::log::Level::Warn,
            LogLevel::Error => tracing_log::log::Level::Error,
        }
    }
}

impl From<tracing_log::log::Level> for LogLevel {
    fn from(log_level: tracing_log::log::Level) -> Self {
        match log_level {
            tracing_log::log::Level::Trace => LogLevel::Trace,
            tracing_log::log::Level::Debug => LogLevel::Debug,
            tracing_log::log::Level::Info => LogLevel::Info,
            tracing_log::log::Level::Warn => LogLevel::Warn,
            tracing_log::log::Level::Error => LogLevel::Error,
        }
    }
}

// pub const WEBVIEW_TARGET: &str = "webview";

// const DEFAULT_MAX_FILE_SIZE: u128 = 40000;
// const DEFAULT_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepOne;
// const DEFAULT_TIMEZONE_STRATEGY: TimezoneStrategy = TimezoneStrategy::UseUtc;
// const DEFAULT_LOG_TARGETS: [Target; 2] = [
//     Target::new(TargetKind::Stdout),
//     Target::new(TargetKind::LogDir { file_name: None }),
// ];

// #[derive(Debug, Clone, Default)]
// pub enum RotationStrategy {
//     #[default]
//     KeepAll,
//     KeepOne,
// }

// #[derive(Debug, Clone, Default)]
// pub enum TimezoneStrategy {
//     #[default]
//     UseUtc,
//     UseLocal,
// }

// /// An enum representing the available targets of the logger.
// pub enum TargetKind {
//     /// Print logs to stdout.
//     Stdout,
//     /// Print logs to stderr.
//     Stderr,
//     /// Write logs to the given directory.
//     ///
//     /// The plugin will ensure the directory exists before writing logs.
//     Folder {
//         path: PathBuf,
//         file_name: Option<String>,
//     },
//     /// Write logs to the OS specific logs directory.
//     ///
//     /// ### Platform-specific
//     ///
//     /// |Platform | Value                                                                                     | Example                                                     |
//     /// | ------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
//     /// | Linux   | `$XDG_DATA_HOME/{bundleIdentifier}/logs` or `$HOME/.local/share/{bundleIdentifier}/logs`  | `/home/alice/.local/share/com.tauri.dev/logs`               |
//     /// | macOS   | `{homeDir}/Library/Logs/{bundleIdentifier}`                                               | `/Users/Alice/Library/Logs/com.tauri.dev`                   |
//     /// | Windows | `{FOLDERID_LocalAppData}/{bundleIdentifier}/logs`                                         | `C:\Users\Alice\AppData\Local\com.tauri.dev\logs`           |
//     LogDir { file_name: Option<String> },
//     /// Forward logs to the webview (via the `log://log` event).
//     ///
//     /// This requires the webview to subscribe to log events, via this plugin's `attachConsole` function.
//     Webview,
// }

// impl TimezoneStrategy {
//     pub fn get_now(&self) -> OffsetDateTime {
//         match self {
//             TimezoneStrategy::UseUtc => OffsetDateTime::now_utc(),
//             TimezoneStrategy::UseLocal => {
//                 OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
//             } // Fallback to UTC since Rust cannot determine local timezone
//         }
//     }
// }

// /// A log target.
// pub struct Target {
//     pub kind: TargetKind,
//     pub filters: Vec<Box<Filter>>,
// }

// impl Target {
//     #[inline]
//     pub const fn new(kind: TargetKind) -> Self {
//         Self {
//             kind,
//             filters: Vec::new(),
//         }
//     }

//     #[inline]
//     pub fn filter<F>(mut self, filter: F) -> Self
//     where
//         F: Fn(&tracing_log::log::Metadata) -> bool + Send + Sync + 'static,
//     {
//         self.filters.push(Box::new(filter));
//         self
//     }
// }
