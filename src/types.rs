//! Configuration types for the tracing plugin.

use std::path::PathBuf;

/// Specifies a log output destination.
///
/// Use these variants to configure where logs should be written. Multiple
/// targets can be combined using [`Builder::target()`](crate::Builder::target) or [`Builder::targets()`](crate::Builder::targets).
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, Target};
///
/// // Log to stdout and webview (default behavior)
/// Builder::new()
///     .targets([Target::Stdout, Target::Webview])
///     .build::<tauri::Wry>();
///
/// // Log to file and webview only (no console)
/// Builder::new()
///     .targets([
///         Target::LogDir { file_name: None },
///         Target::Webview,
///     ])
///     .build::<tauri::Wry>();
///
/// // Log to stderr instead of stdout
/// Builder::new()
///     .clear_targets()
///     .target(Target::Stderr)
///     .target(Target::Webview)
///     .build::<tauri::Wry>();
/// ```
#[derive(Debug, Clone)]
pub enum Target {
    /// Print logs to stdout.
    Stdout,

    /// Print logs to stderr.
    Stderr,

    /// Forward logs to the webview via the `tracing://log` event.
    ///
    /// This allows JavaScript code to receive logs using `attachLogger()`
    /// or `attachConsole()`.
    Webview,

    /// Write logs to the platform-standard log directory.
    ///
    /// Platform log directories:
    /// - **macOS**: `~/Library/Logs/{bundle_identifier}`
    /// - **Linux**: `~/.local/share/{bundle_identifier}/logs`
    /// - **Windows**: `%LOCALAPPDATA%/{bundle_identifier}/logs`
    ///
    /// The `file_name` parameter sets the log file prefix. Defaults to `"app"`
    /// if `None`, producing files like `app.2024-01-15.log`.
    LogDir {
        /// The log file prefix. Defaults to `"app"` if `None`.
        file_name: Option<String>,
    },

    /// Write logs to a custom directory.
    ///
    /// The `file_name` parameter sets the log file prefix. Defaults to `"app"`
    /// if `None`, producing files like `app.2024-01-15.log`.
    Folder {
        /// The directory path to write log files to.
        path: PathBuf,
        /// The log file prefix. Defaults to `"app"` if `None`.
        file_name: Option<String>,
    },
}

/// Time-based rotation period for log files.
///
/// This controls how often new log files are created.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, Rotation};
///
/// Builder::new()
///     .with_file_logging()
///     .with_rotation(Rotation::Daily)  // Create new file each day
///     .build::<tauri::Wry>();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub enum Rotation {
    /// Rotate logs daily. Files are named `app.YYYY-MM-DD.log`.
    #[default]
    Daily,
    /// Rotate logs hourly. Files are named `app.YYYY-MM-DD-HH.log`.
    Hourly,
    /// Rotate logs every minute. Files are named `app.YYYY-MM-DD-HH-MM.log`.
    Minutely,
    /// Never rotate. All logs go to `app.log`.
    Never,
}

/// Retention policy for rotated log files.
///
/// This controls how many old log files are kept when the application starts.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, RotationStrategy};
///
/// Builder::new()
///     .with_file_logging()
///     .with_rotation_strategy(RotationStrategy::KeepSome(7))  // Keep 7 most recent files
///     .build::<tauri::Wry>();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub enum RotationStrategy {
    /// Keep all rotated log files.
    #[default]
    KeepAll,
    /// Keep only the current log file, deleting all previous ones.
    KeepOne,
    /// Keep the N most recent log files.
    KeepSome(u32),
}

/// Log output format style.
///
/// Controls the overall structure and verbosity of log output.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, LogFormat};
///
/// Builder::new()
///     .with_format(LogFormat::Compact)
///     .build::<tauri::Wry>();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub enum LogFormat {
    /// The default format with all information on a single line.
    ///
    /// Output: `2024-01-15T10:30:00.000Z INFO my_app: message field=value`
    #[default]
    Full,

    /// A compact format optimized for shorter line lengths.
    ///
    /// Fields from the current span context are appended to the event fields
    /// rather than displayed in a separate section.
    Compact,

    /// A multi-line, human-readable format for development.
    ///
    /// Includes colorful formatting, indentation, and verbose span information.
    /// Best suited for local development and debugging.
    Pretty,
}

/// Configuration options for log output formatting.
///
/// This struct bundles all the formatting options that control what
/// information is included in log output.
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// The overall format style.
    pub format: LogFormat,
    /// Whether to show the source file path.
    pub file: bool,
    /// Whether to show the source line number.
    pub line_number: bool,
    /// Whether to show thread IDs.
    pub thread_ids: bool,
    /// Whether to show thread names.
    pub thread_names: bool,
    /// Whether to show the log target (module path).
    pub target: bool,
    /// Whether to show the log level.
    pub level: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            format: LogFormat::default(),
            file: false,
            line_number: false,
            thread_ids: false,
            thread_names: false,
            target: true,
            level: true,
        }
    }
}

/// Maximum file size for log rotation.
///
/// Provides convenient constructors for common size units.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, MaxFileSize};
///
/// Builder::new()
///     .with_file_logging()
///     .with_max_file_size(MaxFileSize::mb(10))  // Rotate at 10 MB
///     .build::<tauri::Wry>();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct MaxFileSize(pub(crate) u64);

impl MaxFileSize {
    /// Creates a max file size from raw bytes.
    pub const fn bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Creates a max file size in kilobytes (KB).
    pub const fn kb(kb: u64) -> Self {
        Self(kb * 1024)
    }

    /// Creates a max file size in megabytes (MB).
    pub const fn mb(mb: u64) -> Self {
        Self(mb * 1024 * 1024)
    }

    /// Creates a max file size in gigabytes (GB).
    pub const fn gb(gb: u64) -> Self {
        Self(gb * 1024 * 1024 * 1024)
    }

    /// Returns the size in bytes.
    pub const fn as_bytes(&self) -> u64 {
        self.0
    }
}

impl From<u64> for MaxFileSize {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}

/// Timezone strategy for log timestamps.
///
/// Controls whether log timestamps are displayed in UTC or local time.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, TimezoneStrategy};
///
/// Builder::new()
///     .with_timezone_strategy(TimezoneStrategy::Local)
///     .build::<tauri::Wry>();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub enum TimezoneStrategy {
    /// Use UTC timestamps (e.g., `2024-01-15T14:30:00.000000Z`).
    ///
    /// This is the default and most reliable option.
    #[default]
    Utc,

    /// Use local timestamps with the system's timezone offset.
    ///
    /// The offset is captured when the logger is initialized. If the offset
    /// cannot be determined (e.g., on some Unix systems with multiple threads),
    /// falls back to UTC.
    Local,
}
