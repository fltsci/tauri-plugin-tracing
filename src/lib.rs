//! # Tauri Plugin Tracing
//!
//! A Tauri plugin that integrates the [`tracing`] crate for structured logging
//! in Tauri applications. This plugin bridges logging between the Rust backend
//! and JavaScript frontend, providing call stack information and optional
//! performance timing.
//!
//! ## Features
//!
//! - **`colored`**: Enables colored terminal output using ANSI escape codes
//! - **`specta`**: Enables TypeScript type generation via the `specta` crate
//! - **`timing`**: Enables performance timing with `time()` and `timeEnd()` APIs
//!
//! ## Usage
//!
//! By default, this plugin does **not** set up a global tracing subscriber,
//! following the convention that libraries should not set globals. You compose
//! your own subscriber using [`WebviewLayer`] to forward logs to the frontend:
//!
//! ```rust,no_run
//! # use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
//! # use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
//! let tracing_builder = Builder::new()
//!     .with_max_level(LevelFilter::DEBUG)
//!     .with_target("hyper", LevelFilter::WARN);
//! let filter = tracing_builder.build_filter();
//!
//! tauri::Builder::default()
//!     .plugin(tracing_builder.build())
//!     .setup(move |app| {
//!         let subscriber = Registry::default()
//!             .with(fmt::layer())
//!             .with(WebviewLayer::new(app.handle().clone()))
//!             .with(filter);
//!         tracing::subscriber::set_global_default(subscriber)?;
//!         Ok(())
//!     });
//!     // .run(tauri::generate_context!())
//! ```
//!
//! ## Quick Start
//!
//! For simple applications, use [`Builder::with_default_subscriber()`] to let
//! the plugin handle all tracing setup:
//!
//! ```rust,no_run
//! # use tauri_plugin_tracing::{Builder, LevelFilter};
//! tauri::Builder::default()
//!     .plugin(
//!         Builder::new()
//!             .with_max_level(LevelFilter::DEBUG)
//!             .with_default_subscriber()  // Let plugin set up tracing
//!             .build(),
//!     );
//!     // .run(tauri::generate_context!())
//! ```
//!
//! ## File Logging
//!
//! File logging requires [`Builder::with_default_subscriber()`]:
//!
//! ```rust,no_run
//! # use tauri_plugin_tracing::{Builder, LevelFilter};
//! Builder::new()
//!     .with_max_level(LevelFilter::DEBUG)
//!     .with_file_logging()
//!     .with_default_subscriber()
//!     .build::<tauri::Wry>();
//! ```
//!
//! Log files rotate daily and are written to:
//! - **macOS**: `~/Library/Logs/{bundle_identifier}/app.YYYY-MM-DD.log`
//! - **Linux**: `~/.local/share/{bundle_identifier}/logs/app.YYYY-MM-DD.log`
//! - **Windows**: `%LOCALAPPDATA%/{bundle_identifier}/logs/app.YYYY-MM-DD.log`
//!
//! ## JavaScript API
//!
//! ```javascript
//! import { trace, debug, info, warn, error } from '@fltsci/tauri-plugin-tracing';
//!
//! info('Application started');
//! debug('Debug information', { key: 'value' });
//! error('Something went wrong');
//! ```

mod callstack;
#[cfg(feature = "timing")]
mod timing;

use std::path::PathBuf;
use tauri::plugin::{self, TauriPlugin};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    Layer, Registry,
    filter::{Targets, filter_fn},
    fmt::{self, SubscriberBuilder},
    layer::SubscriberExt,
};

/// A boxed filter function for metadata-based log filtering.
///
/// This type alias represents a filter that examines event metadata to determine
/// whether a log should be emitted. The function receives a reference to the
/// metadata and returns `true` if the log should be included.
pub type FilterFn = Box<dyn Fn(&tracing::Metadata<'_>) -> bool + Send + Sync>;

/// A boxed tracing layer that can be added to the default subscriber.
///
/// Use this type with [`Builder::with_layer()`] to add custom tracing layers
/// (e.g., for OpenTelemetry, Sentry, or custom logging integrations) to the
/// plugin-managed subscriber.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, BoxedLayer};
/// use tracing_subscriber::Layer;
///
/// // Create a custom layer (e.g., from another crate) and box it
/// let my_layer: BoxedLayer = tracing_subscriber::fmt::layer().boxed();
///
/// Builder::new()
///     .with_layer(my_layer)
///     .with_default_subscriber()
///     .build::<tauri::Wry>();
/// ```
pub type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub use callstack::*;
#[cfg(feature = "timing")]
pub use timing::*;

/// Extension trait for Tauri managers that provides timing functionality.
///
/// This trait is automatically implemented for all types that implement
/// [`tauri::Manager`] when the `timing` feature is enabled.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "timing")]
/// # async fn example(app: tauri::AppHandle) {
/// use tauri_plugin_tracing::LoggerExt;
///
/// // In a Tauri command:
/// app.time("my_operation".into()).await;
/// // ... perform operation ...
/// app.time_end("my_operation".into(), None).await;
/// # }
/// ```
#[async_trait::async_trait]
pub trait LoggerExt<R: Runtime> {
    /// Starts a timer with the given label.
    ///
    /// The timer can be stopped later with [`time_end`](Self::time_end) using the same label.
    #[cfg(feature = "timing")]
    async fn time(&self, label: compact_str::CompactString);

    /// Stops a timer and logs the elapsed time.
    ///
    /// If a timer with the given label exists, logs the elapsed time in milliseconds.
    /// If no timer with that label exists, logs a warning.
    #[cfg(feature = "timing")]
    async fn time_end(&self, label: compact_str::CompactString, call_stack: Option<String>);
}

/// Re-export of the [`tracing`] crate for convenience.
pub use tracing;
/// Re-export of the [`tracing_appender`] crate for file logging configuration.
pub use tracing_appender;
/// Re-export of the [`tracing_subscriber`] crate for subscriber configuration.
pub use tracing_subscriber;

/// Re-export of [`tracing_subscriber::filter::LevelFilter`] for configuring log levels.
pub use tracing_subscriber::filter::LevelFilter;

use serde::ser::Serializer;
use tracing::Level;

#[cfg(target_os = "ios")]
mod ios {
    swift_rs::swift!(pub fn tauri_log(
      level: u8, message: *const std::ffi::c_void
    ));
}

/// A specialized [`Result`](std::result::Result) type for tracing plugin operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the tracing plugin.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Error {
    /// An error from the Tauri runtime.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    Tauri(#[from] tauri::Error),

    /// An I/O error, typically from file operations.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    Io(#[from] std::io::Error),

    /// An error formatting a timestamp.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    TimeFormat(#[from] time::error::Format),

    /// An invalid time format description was provided.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),

    /// The internal logger was not initialized.
    #[error("Internal logger disabled and cannot be acquired or attached")]
    LoggerNotInitialized,

    /// Failed to set the global default subscriber.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),

    /// The requested feature is not yet implemented.
    #[error("Not implemented")]
    NotImplemented,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// Specifies a log output destination.
///
/// Use these variants to configure where logs should be written. Multiple
/// targets can be combined using [`Builder::target()`] or [`Builder::targets()`].
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
pub struct MaxFileSize(u64);

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

/// Stores the WorkerGuard to ensure logs are flushed on shutdown.
/// This must be kept alive for the lifetime of the application.
struct LogGuard(#[allow(dead_code)] Option<WorkerGuard>);

/// A writer wrapper that strips ANSI escape codes from all output.
///
/// This is used for file logging to ensure clean output even when stdout
/// layers use ANSI colors. Due to how `tracing_subscriber` shares span field
/// formatting between layers, ANSI codes from one layer can leak into others.
/// This wrapper strips those codes at write time.
///
/// Uses a zero-copy fast path when no ANSI codes are present. Thread-safe
/// via internal Mutex.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::StripAnsiWriter;
/// use tauri_plugin_tracing::tracing_appender::non_blocking;
/// use tauri_plugin_tracing::tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
///
/// let file_appender = tracing_appender::rolling::daily("/tmp/logs", "app.log");
/// let (non_blocking, _guard) = non_blocking(file_appender);
///
/// tracing_subscriber::registry()
///     .with(fmt::layer())  // stdout with ANSI
///     .with(fmt::layer().with_writer(StripAnsiWriter::new(non_blocking)).with_ansi(false))
///     .init();
/// ```
pub struct StripAnsiWriter<W> {
    inner: std::sync::Mutex<W>,
}

impl<W> StripAnsiWriter<W> {
    /// Creates a new `StripAnsiWriter` that wraps the given writer.
    pub fn new(inner: W) -> Self {
        Self {
            inner: std::sync::Mutex::new(inner),
        }
    }
}

/// Strips ANSI escape codes from input and writes to output.
/// Returns the number of bytes from input that were processed.
fn strip_ansi_and_write<W: std::io::Write>(writer: &mut W, buf: &[u8]) -> std::io::Result<usize> {
    let input_len = buf.len();

    // Fast path: use memchr to check for ESC byte. If none, write directly.
    let Some(first_esc) = memchr::memchr(0x1b, buf) else {
        writer.write_all(buf)?;
        return Ok(input_len);
    };

    // Slow path: ANSI codes present, need to strip them
    // Pre-allocate with capacity for worst case (all kept)
    let mut output = Vec::with_capacity(input_len);

    // Copy everything before first ESC
    output.extend_from_slice(&buf[..first_esc]);
    let mut i = first_esc;

    while i < buf.len() {
        if buf[i] == 0x1b && i + 1 < buf.len() && buf[i + 1] == b'[' {
            // Found ESC[, skip the SGR sequence
            i += 2;
            while i < buf.len() {
                let c = buf[i];
                i += 1;
                if c == b'm' {
                    break;
                }
                if !c.is_ascii_digit() && c != b';' {
                    break;
                }
            }
        } else {
            output.push(buf[i]);
            i += 1;
        }
    }

    writer.write_all(&output)?;
    Ok(input_len)
}

/// A writer handle returned by [`StripAnsiWriter::make_writer`].
///
/// This type implements [`std::io::Write`] and strips ANSI codes during writes.
pub struct StripAnsiWriterGuard<'a, W> {
    guard: std::sync::MutexGuard<'a, W>,
}

impl<W: std::io::Write> std::io::Write for StripAnsiWriterGuard<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        strip_ansi_and_write(&mut *self.guard, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.guard.flush()
    }
}

// Implement MakeWriter so this can be used with fmt::layer().with_writer()
impl<'a, W: std::io::Write + 'a> tracing_subscriber::fmt::MakeWriter<'a> for StripAnsiWriter<W> {
    type Writer = StripAnsiWriterGuard<'a, W>;

    fn make_writer(&'a self) -> Self::Writer {
        StripAnsiWriterGuard {
            guard: self.inner.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }
}

/// A log message consisting of one or more string parts.
///
/// This type wraps a `Vec<String>` to allow logging multiple values in a single call.
/// When displayed, the parts are joined with ", ".
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

/// Payload for a log record, used when emitting events to the webview.
#[derive(Debug, Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RecordPayload {
    /// The formatted log message.
    pub message: String,
    /// The severity level of the log.
    pub level: LogLevel,
}

/// A tracing layer that emits log events to the webview via Tauri events.
///
/// This layer intercepts all log events and forwards them to the frontend
/// via the `tracing://log` event, allowing JavaScript code to receive
/// logs using `attachLogger()` or `attachConsole()`.
///
/// # Example
///
/// By default, the plugin does not set up a global subscriber. Use this layer
/// when composing your own subscriber:
///
/// ```rust,no_run
/// # use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
/// # use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
/// let builder = Builder::new()
///     .with_max_level(LevelFilter::DEBUG);
/// let filter = builder.build_filter();
///
/// tauri::Builder::default()
///     .plugin(builder.build())
///     .setup(move |app| {
///         let subscriber = Registry::default()
///             .with(fmt::layer())
///             .with(WebviewLayer::new(app.handle().clone()))
///             .with(filter);
///         tracing::subscriber::set_global_default(subscriber)?;
///         Ok(())
///     });
///     // .run(tauri::generate_context!())
/// ```
pub struct WebviewLayer<R: Runtime> {
    app_handle: AppHandle<R>,
}

impl<R: Runtime> WebviewLayer<R> {
    /// Creates a new WebviewLayer that forwards log events to the given app handle.
    ///
    /// Events are emitted via the `tracing://log` event channel.
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

impl<S, R: Runtime> Layer<S> for WebviewLayer<R>
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let level: LogLevel = (*event.metadata().level()).into();
        let payload = RecordPayload {
            message: visitor.message,
            level,
        };

        let _ = self.app_handle.emit("tracing://log", payload);
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" || self.message.is_empty() {
            self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" || self.message.is_empty() {
            self.message = value.to_string();
        }
    }
}

/// An enum representing the available verbosity levels of the logger.
///
/// It is very similar to `log::Level`, but serializes to unsigned ints instead of strings.
///
/// # Examples
///
/// ```
/// use tauri_plugin_tracing::LogLevel;
///
/// // Default is Info
/// assert!(matches!(LogLevel::default(), LogLevel::Info));
///
/// // Convert to tracing::Level
/// let level: tracing::Level = LogLevel::Debug.into();
/// assert_eq!(level, tracing::Level::DEBUG);
///
/// // Convert from tracing::Level
/// let log_level: LogLevel = tracing::Level::WARN.into();
/// assert!(matches!(log_level, LogLevel::Warn));
/// ```
#[derive(Debug, Clone, Deserialize_repr, Serialize_repr, Default)]
#[repr(u16)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

#[tauri::command]
#[tracing::instrument(skip_all, fields(w = %CallStackLine::from(webview_window.label())))]
fn log<R: Runtime>(
    webview_window: tauri::WebviewWindow<R>,
    level: LogLevel,
    message: LogMessage,
    call_stack: Option<&str>,
) {
    let stack = CallStack::from(call_stack);
    let loc = match level {
        LogLevel::Trace => stack.location(),
        LogLevel::Debug => stack.path(),
        LogLevel::Info => stack.file_name(),
        LogLevel::Warn => stack.path(),
        LogLevel::Error => stack.location(),
    };
    macro_rules! emit_event {
        ($level:expr) => {
            tracing::event!(
                target: "",
                $level,
                %message,
                "" = %loc,
            )
        };
    }
    match level {
        LogLevel::Trace => emit_event!(Level::TRACE),
        LogLevel::Debug => emit_event!(Level::DEBUG),
        LogLevel::Info => emit_event!(Level::INFO),
        LogLevel::Warn => emit_event!(Level::WARN),
        LogLevel::Error => emit_event!(Level::ERROR),
    }
}

#[cfg(feature = "timing")]
#[tauri::command]
async fn time<R: Runtime>(app: AppHandle<R>, label: String) {
    use compact_str::ToCompactString;
    app.time(label.to_compact_string()).await;
}

#[cfg(feature = "timing")]
#[tauri::command]
async fn time_end<R: Runtime>(app: AppHandle<R>, label: String, call_stack: Option<String>) {
    use compact_str::ToCompactString;
    app.time_end(label.to_compact_string(), call_stack).await;
}

/// Builder for configuring and creating the tracing plugin.
///
/// Use this builder to customize logging behavior before registering the plugin
/// with your Tauri application.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::{Builder, LevelFilter};
///
/// let plugin = Builder::new()
///     .with_max_level(LevelFilter::DEBUG)
///     .with_target("hyper", LevelFilter::WARN)  // Reduce noise from hyper
///     .with_target("my_app", LevelFilter::TRACE)  // Verbose logging for your app
///     .build::<tauri::Wry>();
/// ```
pub struct Builder {
    builder: SubscriberBuilder,
    log_level: LevelFilter,
    filter: Targets,
    custom_filter: Option<FilterFn>,
    custom_layer: Option<BoxedLayer>,
    targets: Vec<Target>,
    rotation: Rotation,
    rotation_strategy: RotationStrategy,
    max_file_size: Option<MaxFileSize>,
    timezone_strategy: TimezoneStrategy,
    log_format: LogFormat,
    show_file: bool,
    show_line_number: bool,
    show_thread_ids: bool,
    show_thread_names: bool,
    show_target: bool,
    show_level: bool,
    set_default_subscriber: bool,
    #[cfg(feature = "colored")]
    use_colors: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            builder: SubscriberBuilder::default(),
            log_level: LevelFilter::WARN,
            filter: Targets::default(),
            custom_filter: None,
            custom_layer: None,
            targets: vec![Target::Stdout, Target::Webview],
            rotation: Rotation::default(),
            rotation_strategy: RotationStrategy::default(),
            max_file_size: None,
            timezone_strategy: TimezoneStrategy::default(),
            log_format: LogFormat::default(),
            show_file: false,
            show_line_number: false,
            show_thread_ids: false,
            show_thread_names: false,
            show_target: true,
            show_level: true,
            set_default_subscriber: false,
            #[cfg(feature = "colored")]
            use_colors: false,
        }
    }
}

impl Builder {
    /// Creates a new builder with default settings.
    ///
    /// The default log level is [`LevelFilter::WARN`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the maximum log level.
    ///
    /// Events more verbose than this level will be filtered out.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::{Builder, LevelFilter};
    /// Builder::new().with_max_level(LevelFilter::DEBUG);
    /// ```
    pub fn with_max_level(mut self, max_level: LevelFilter) -> Self {
        self.log_level = max_level;
        self.builder = self.builder.with_max_level(max_level);
        self
    }

    /// Sets the log level for a specific target (module path).
    ///
    /// This allows fine-grained control over logging verbosity for different
    /// parts of your application or dependencies.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::{Builder, LevelFilter};
    /// Builder::new()
    ///     .with_max_level(LevelFilter::INFO)
    ///     .with_target("my_app::database", LevelFilter::DEBUG)
    ///     .with_target("hyper", LevelFilter::WARN);
    /// ```
    pub fn with_target(mut self, target: &str, level: LevelFilter) -> Self {
        self.filter = self.filter.with_target(target, level);
        self
    }

    /// Sets a custom filter function for metadata-based log filtering.
    ///
    /// The filter function receives the metadata for each log event and returns
    /// `true` if the event should be logged. This filter is applied in addition
    /// to the level and target filters configured via [`with_max_level()`](Self::with_max_level)
    /// and [`with_target()`](Self::with_target).
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    /// For custom subscribers, use [`tracing_subscriber::filter::filter_fn()`] directly.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::Builder;
    ///
    /// // Filter out logs from a specific module
    /// Builder::new()
    ///     .filter(|metadata| {
    ///         metadata.target() != "noisy_crate::spammy_module"
    ///     })
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    ///
    /// // Only log events (not spans)
    /// Builder::new()
    ///     .filter(|metadata| metadata.is_event())
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&tracing::Metadata<'_>) -> bool + Send + Sync + 'static,
    {
        self.custom_filter = Some(Box::new(filter));
        self
    }

    /// Adds a custom tracing layer to the subscriber.
    ///
    /// Use this to integrate additional tracing functionality (e.g., OpenTelemetry,
    /// Sentry, custom metrics) with the plugin-managed subscriber.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// Note: Only one custom layer is supported. Calling this multiple times will
    /// replace the previous layer. To use multiple custom layers, compose them
    /// with [`tracing_subscriber::layer::Layered`] before passing to this method.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::Builder;
    /// use tracing_subscriber::Layer;
    ///
    /// // Add a custom layer (e.g., a secondary fmt layer or OpenTelemetry)
    /// let custom_layer = tracing_subscriber::fmt::layer().boxed();
    ///
    /// Builder::new()
    ///     .with_layer(custom_layer)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_layer(mut self, layer: BoxedLayer) -> Self {
        self.custom_layer = Some(layer);
        self
    }

    /// Enables colored output in the terminal.
    ///
    /// This adds ANSI color codes to log level indicators.
    /// Only available when the `colored` feature is enabled.
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    #[cfg(feature = "colored")]
    pub fn with_colors(mut self) -> Self {
        self.builder = self.builder.with_ansi(true);
        self.use_colors = true;
        self
    }

    /// Enables file logging to the platform-standard log directory.
    ///
    /// Log files rotate daily with the naming pattern `app.YYYY-MM-DD.log`.
    ///
    /// Platform log directories:
    /// - **macOS**: `~/Library/Logs/{bundle_identifier}`
    /// - **Linux**: `~/.local/share/{bundle_identifier}/logs`
    /// - **Windows**: `%LOCALAPPDATA%/{bundle_identifier}/logs`
    ///
    /// This is a convenience method equivalent to calling
    /// `.target(Target::LogDir { file_name: None })`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::{Builder, LevelFilter};
    /// Builder::new()
    ///     .with_max_level(LevelFilter::DEBUG)
    ///     .with_file_logging()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_file_logging(self) -> Self {
        self.target(Target::LogDir { file_name: None })
    }

    /// Sets the rotation period for log files.
    ///
    /// This controls how often new log files are created. Only applies when
    /// file logging is enabled.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, Rotation};
    ///
    /// Builder::new()
    ///     .with_file_logging()
    ///     .with_rotation(Rotation::Hourly)  // Rotate every hour
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the retention strategy for rotated log files.
    ///
    /// This controls how many old log files are kept. Cleanup happens when
    /// the application starts.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, RotationStrategy};
    ///
    /// Builder::new()
    ///     .with_file_logging()
    ///     .with_rotation_strategy(RotationStrategy::KeepSome(7))  // Keep 7 files
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_rotation_strategy(mut self, strategy: RotationStrategy) -> Self {
        self.rotation_strategy = strategy;
        self
    }

    /// Sets the maximum file size before rotating.
    ///
    /// When set, log files will rotate when they reach this size, in addition
    /// to any time-based rotation configured via [`with_rotation()`](Self::with_rotation).
    ///
    /// Use [`MaxFileSize`] for convenient size specification:
    /// - `MaxFileSize::kb(100)` - 100 kilobytes
    /// - `MaxFileSize::mb(10)` - 10 megabytes
    /// - `MaxFileSize::gb(1)` - 1 gigabyte
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, MaxFileSize};
    ///
    /// // Rotate when file reaches 10 MB
    /// Builder::new()
    ///     .with_file_logging()
    ///     .with_max_file_size(MaxFileSize::mb(10))
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_max_file_size(mut self, size: MaxFileSize) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// Sets the timezone strategy for log timestamps.
    ///
    /// Controls whether timestamps are displayed in UTC or local time.
    /// The default is [`TimezoneStrategy::Utc`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, TimezoneStrategy};
    ///
    /// // Use local time for timestamps
    /// Builder::new()
    ///     .with_timezone_strategy(TimezoneStrategy::Local)
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_timezone_strategy(mut self, strategy: TimezoneStrategy) -> Self {
        self.timezone_strategy = strategy;
        self
    }

    /// Sets the log output format style.
    ///
    /// Controls the overall structure of log output. The default is [`LogFormat::Full`].
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, LogFormat};
    ///
    /// // Use compact format for shorter lines
    /// Builder::new()
    ///     .with_format(LogFormat::Compact)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.log_format = format;
        self
    }

    /// Sets whether to include the source file path in log output.
    ///
    /// When enabled, logs will show which file the log event originated from.
    /// Default is `false`.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// Builder::new()
    ///     .with_file(true)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_file(mut self, show: bool) -> Self {
        self.show_file = show;
        self
    }

    /// Sets whether to include the source line number in log output.
    ///
    /// When enabled, logs will show which line number the log event originated from.
    /// Default is `false`.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// Builder::new()
    ///     .with_line_number(true)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_line_number(mut self, show: bool) -> Self {
        self.show_line_number = show;
        self
    }

    /// Sets whether to include the current thread ID in log output.
    ///
    /// When enabled, logs will show the ID of the thread that emitted the event.
    /// Default is `false`.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// Builder::new()
    ///     .with_thread_ids(true)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_thread_ids(mut self, show: bool) -> Self {
        self.show_thread_ids = show;
        self
    }

    /// Sets whether to include the current thread name in log output.
    ///
    /// When enabled, logs will show the name of the thread that emitted the event.
    /// Default is `false`.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// Builder::new()
    ///     .with_thread_names(true)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_thread_names(mut self, show: bool) -> Self {
        self.show_thread_names = show;
        self
    }

    /// Sets whether to include the log target (module path) in log output.
    ///
    /// When enabled, logs will show which module/target emitted the event.
    /// Default is `true`.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// // Disable target display for cleaner output
    /// Builder::new()
    ///     .with_target_display(false)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_target_display(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }

    /// Sets whether to include the log level in log output.
    ///
    /// When enabled, logs will show the severity level (TRACE, DEBUG, INFO, etc.).
    /// Default is `true`.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// // Disable level display
    /// Builder::new()
    ///     .with_level(false)
    ///     .with_default_subscriber()
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn with_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    /// Adds a log output target.
    ///
    /// By default, logs are sent to [`Target::Stdout`] and [`Target::Webview`].
    /// Use this method to add additional targets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// // Add file logging to the default targets
    /// Builder::new()
    ///     .target(Target::LogDir { file_name: None })
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn target(mut self, target: Target) -> Self {
        self.targets.push(target);
        self
    }

    /// Sets the log output targets, replacing any previously configured targets.
    ///
    /// By default, logs are sent to [`Target::Stdout`] and [`Target::Webview`].
    /// Use this method to completely replace the default targets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// // Log only to file and webview (no stdout)
    /// Builder::new()
    ///     .targets([
    ///         Target::LogDir { file_name: None },
    ///         Target::Webview,
    ///     ])
    ///     .build::<tauri::Wry>();
    ///
    /// // Log only to stderr
    /// Builder::new()
    ///     .targets([Target::Stderr])
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn targets(mut self, targets: impl IntoIterator<Item = Target>) -> Self {
        self.targets = targets.into_iter().collect();
        self
    }

    /// Removes all configured log targets.
    ///
    /// Use this followed by [`target()`](Self::target) to build a custom set
    /// of targets from scratch.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// // Start fresh and only log to webview
    /// Builder::new()
    ///     .clear_targets()
    ///     .target(Target::Webview)
    ///     .build::<tauri::Wry>();
    /// ```
    pub fn clear_targets(mut self) -> Self {
        self.targets.clear();
        self
    }

    /// Enables the plugin to set up and register the global tracing subscriber.
    ///
    /// By default, this plugin does **not** call [`tracing::subscriber::set_global_default()`],
    /// following the convention that libraries should not set globals. This allows your
    /// application to compose its own subscriber with layers from multiple crates.
    ///
    /// Call this method if you want the plugin to handle all tracing setup for you,
    /// using the configuration from this builder (log levels, targets, file logging, etc.).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::{Builder, LevelFilter};
    /// // Let the plugin set up everything
    /// tauri::Builder::default()
    ///     .plugin(
    ///         Builder::new()
    ///             .with_max_level(LevelFilter::DEBUG)
    ///             .with_file_logging()
    ///             .with_default_subscriber()  // Opt-in to global subscriber
    ///             .build()
    ///     );
    ///     // .run(tauri::generate_context!())
    /// ```
    pub fn with_default_subscriber(mut self) -> Self {
        self.set_default_subscriber = true;
        self
    }

    /// Returns the configured log output targets.
    ///
    /// Use this when setting up your own subscriber to determine which
    /// layers to include based on the configured targets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// let builder = Builder::new()
    ///     .target(Target::LogDir { file_name: None });
    ///
    /// for target in builder.configured_targets() {
    ///     match target {
    ///         Target::Stdout => { /* add stdout layer */ }
    ///         Target::Stderr => { /* add stderr layer */ }
    ///         Target::Webview => { /* add WebviewLayer */ }
    ///         Target::LogDir { .. } | Target::Folder { .. } => { /* add file layer */ }
    ///     }
    /// }
    /// ```
    pub fn configured_targets(&self) -> &[Target] {
        &self.targets
    }

    /// Returns the configured rotation period for file logging.
    pub fn configured_rotation(&self) -> Rotation {
        self.rotation
    }

    /// Returns the configured rotation strategy for file logging.
    pub fn configured_rotation_strategy(&self) -> RotationStrategy {
        self.rotation_strategy
    }

    /// Returns the configured maximum file size for rotation, if set.
    pub fn configured_max_file_size(&self) -> Option<MaxFileSize> {
        self.max_file_size
    }

    /// Returns the configured timezone strategy for timestamps.
    pub fn configured_timezone_strategy(&self) -> TimezoneStrategy {
        self.timezone_strategy
    }

    /// Returns the configured log format style.
    pub fn configured_format(&self) -> LogFormat {
        self.log_format
    }

    /// Returns the configured format options.
    pub fn configured_format_options(&self) -> FormatOptions {
        FormatOptions {
            format: self.log_format,
            file: self.show_file,
            line_number: self.show_line_number,
            thread_ids: self.show_thread_ids,
            thread_names: self.show_thread_names,
            target: self.show_target,
            level: self.show_level,
        }
    }

    /// Returns the configured filter based on log level and per-target settings.
    ///
    /// Use this when setting up your own subscriber to apply the same filtering
    /// configured via [`with_max_level()`](Self::with_max_level) and
    /// [`with_target()`](Self::with_target).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
    /// # use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
    /// let builder = Builder::new()
    ///     .with_max_level(LevelFilter::DEBUG)
    ///     .with_target("hyper", LevelFilter::WARN);
    ///
    /// let filter = builder.build_filter();
    ///
    /// tauri::Builder::default()
    ///     .plugin(builder.build())
    ///     .setup(move |app| {
    ///         let subscriber = Registry::default()
    ///             .with(fmt::layer())
    ///             .with(WebviewLayer::new(app.handle().clone()))
    ///             .with(filter);
    ///         tracing::subscriber::set_global_default(subscriber)?;
    ///         Ok(())
    ///     });
    ///     // .run(tauri::generate_context!())
    /// ```
    pub fn build_filter(&self) -> Targets {
        self.filter.clone().with_default(self.log_level)
    }

    #[cfg(feature = "timing")]
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing")
            .invoke_handler(tauri::generate_handler![log, time, time_end])
    }

    #[cfg(not(feature = "timing"))]
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![log,])
    }

    /// Builds and returns the configured Tauri plugin.
    ///
    /// This consumes the builder and returns a [`TauriPlugin`] that can be
    /// registered with your Tauri application.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use tauri_plugin_tracing::Builder;
    /// tauri::Builder::default()
    ///     .plugin(Builder::new().build());
    ///     // .run(tauri::generate_context!())
    /// ```
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let log_level = self.log_level;
        let filter = self.filter;
        let custom_filter = self.custom_filter;
        let custom_layer = self.custom_layer;
        let targets = self.targets;
        let rotation = self.rotation;
        let rotation_strategy = self.rotation_strategy;
        let max_file_size = self.max_file_size;
        let timezone_strategy = self.timezone_strategy;
        let format_options = FormatOptions {
            format: self.log_format,
            file: self.show_file,
            line_number: self.show_line_number,
            thread_ids: self.show_thread_ids,
            thread_names: self.show_thread_names,
            target: self.show_target,
            level: self.show_level,
        };
        let set_default_subscriber = self.set_default_subscriber;

        #[cfg(feature = "colored")]
        let use_colors = self.use_colors;

        Self::plugin_builder()
            .setup(move |app, _api| {
                #[cfg(feature = "timing")]
                setup_timings(app);

                #[cfg(desktop)]
                if set_default_subscriber {
                    let guard = acquire_logger(
                        app,
                        log_level,
                        filter,
                        custom_filter,
                        custom_layer,
                        &targets,
                        rotation,
                        rotation_strategy,
                        max_file_size,
                        timezone_strategy,
                        format_options,
                        #[cfg(feature = "colored")]
                        use_colors,
                    )?;

                    // Store the guard in Tauri's state management to ensure logs flush on shutdown
                    if guard.is_some() {
                        app.manage(LogGuard(guard));
                    }
                }

                Ok(())
            })
            .build()
    }
}

/// Initializes the timing state for the application.
#[cfg(feature = "timing")]
fn setup_timings<R: Runtime>(app: &AppHandle<R>) {
    let timings = Timings::default();
    app.manage(timings);
}

/// Configuration for a file logging target.
struct FileTargetConfig {
    log_dir: PathBuf,
    file_name: String,
}

/// Resolves file target configuration from a Target.
fn resolve_file_target<R: Runtime>(
    app_handle: &AppHandle<R>,
    target: &Target,
) -> Result<Option<FileTargetConfig>> {
    match target {
        Target::LogDir { file_name } => {
            let log_dir = app_handle.path().app_log_dir()?;
            std::fs::create_dir_all(&log_dir)?;
            Ok(Some(FileTargetConfig {
                log_dir,
                file_name: file_name.clone().unwrap_or_else(|| "app".to_string()),
            }))
        }
        Target::Folder { path, file_name } => {
            std::fs::create_dir_all(path)?;
            Ok(Some(FileTargetConfig {
                log_dir: path.clone(),
                file_name: file_name.clone().unwrap_or_else(|| "app".to_string()),
            }))
        }
        _ => Ok(None),
    }
}

/// Cleans up old log files based on the retention strategy.
fn cleanup_old_logs(
    log_dir: &std::path::Path,
    file_prefix: &str,
    strategy: RotationStrategy,
) -> Result<()> {
    match strategy {
        RotationStrategy::KeepAll => Ok(()),
        RotationStrategy::KeepOne => cleanup_logs_keeping(log_dir, file_prefix, 1),
        RotationStrategy::KeepSome(n) => cleanup_logs_keeping(log_dir, file_prefix, n as usize),
    }
}

/// Helper to delete old log files, keeping only the most recent `keep` files.
fn cleanup_logs_keeping(log_dir: &std::path::Path, file_prefix: &str, keep: usize) -> Result<()> {
    let prefix_with_dot = format!("{}.", file_prefix);
    let mut log_files: Vec<_> = std::fs::read_dir(log_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(&prefix_with_dot) && name.ends_with(".log"))
        })
        .collect();

    // Sort by filename (which includes date) in descending order (newest first)
    log_files.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));

    // Delete all but the most recent `keep` files
    for entry in log_files.into_iter().skip(keep) {
        if let Err(e) = std::fs::remove_file(entry.path()) {
            tracing::warn!("Failed to remove old log file {:?}: {}", entry.path(), e);
        }
    }

    Ok(())
}

/// Sets up the tracing subscriber based on configured targets.
#[cfg(desktop)]
#[allow(clippy::too_many_arguments)]
fn acquire_logger<R: Runtime>(
    app_handle: &AppHandle<R>,
    log_level: LevelFilter,
    filter: Targets,
    custom_filter: Option<FilterFn>,
    custom_layer: Option<BoxedLayer>,
    targets: &[Target],
    rotation: Rotation,
    rotation_strategy: RotationStrategy,
    max_file_size: Option<MaxFileSize>,
    timezone_strategy: TimezoneStrategy,
    format_options: FormatOptions,
    #[cfg(feature = "colored")] use_colors: bool,
) -> Result<Option<WorkerGuard>> {
    use std::io;
    use tracing_subscriber::fmt::time::OffsetTime;

    let filter_with_default = filter.with_default(log_level);

    // Determine which targets are enabled
    let has_stdout = targets.iter().any(|t| matches!(t, Target::Stdout));
    let has_stderr = targets.iter().any(|t| matches!(t, Target::Stderr));
    let has_webview = targets.iter().any(|t| matches!(t, Target::Webview));

    // Find file target (only first one is used)
    let file_config = targets
        .iter()
        .find_map(|t| resolve_file_target(app_handle, t).transpose())
        .transpose()?;

    // Determine if ANSI should be enabled for stdout/stderr.
    // File output uses StripAnsiWriter to strip ANSI codes, so stdout can use colors.
    #[cfg(feature = "colored")]
    let use_ansi = use_colors;
    #[cfg(not(feature = "colored"))]
    let use_ansi = false;

    // Helper to create timer based on timezone strategy
    let make_timer = || match timezone_strategy {
        TimezoneStrategy::Utc => OffsetTime::new(
            time::UtcOffset::UTC,
            time::format_description::well_known::Rfc3339,
        ),
        TimezoneStrategy::Local => time::UtcOffset::current_local_offset()
            .map(|offset| OffsetTime::new(offset, time::format_description::well_known::Rfc3339))
            .unwrap_or_else(|_| {
                OffsetTime::new(
                    time::UtcOffset::UTC,
                    time::format_description::well_known::Rfc3339,
                )
            }),
    };

    // Macro to create a formatted layer with the appropriate format style.
    // This is needed because .compact() and .pretty() return different types.
    macro_rules! make_layer {
        ($layer:expr, $format:expr) => {
            match $format {
                LogFormat::Full => $layer.boxed(),
                LogFormat::Compact => $layer.compact().boxed(),
                LogFormat::Pretty => $layer.pretty().boxed(),
            }
        };
    }

    // Create optional layers based on targets
    let stdout_layer = if has_stdout {
        let layer = fmt::layer()
            .with_timer(make_timer())
            .with_ansi(use_ansi)
            .with_file(format_options.file)
            .with_line_number(format_options.line_number)
            .with_thread_ids(format_options.thread_ids)
            .with_thread_names(format_options.thread_names)
            .with_target(format_options.target)
            .with_level(format_options.level);
        Some(make_layer!(layer, format_options.format))
    } else {
        None
    };

    let stderr_layer = if has_stderr {
        let layer = fmt::layer()
            .with_timer(make_timer())
            .with_ansi(use_ansi)
            .with_file(format_options.file)
            .with_line_number(format_options.line_number)
            .with_thread_ids(format_options.thread_ids)
            .with_thread_names(format_options.thread_names)
            .with_target(format_options.target)
            .with_level(format_options.level)
            .with_writer(io::stderr);
        Some(make_layer!(layer, format_options.format))
    } else {
        None
    };

    let webview_layer = if has_webview {
        Some(WebviewLayer::new(app_handle.clone()))
    } else {
        None
    };

    // Set up file logging if configured
    let (file_layer, guard) = if let Some(config) = file_config {
        // Note: cleanup_old_logs only works reliably with time-based rotation
        // When using size-based rotation, files have numeric suffixes that may not sort correctly
        if max_file_size.is_none() {
            cleanup_old_logs(&config.log_dir, &config.file_name, rotation_strategy)?;
        }

        // Use rolling-file crate when max_file_size is set (supports both size and time-based rotation)
        // Otherwise use tracing-appender (time-based only)
        if let Some(max_size) = max_file_size {
            use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};

            // Build rolling condition with both time and size triggers
            let mut condition = RollingConditionBasic::new();
            condition = match rotation {
                Rotation::Daily => condition.daily(),
                Rotation::Hourly => condition.hourly(),
                Rotation::Minutely => condition, // rolling-file doesn't have minutely, use size only
                Rotation::Never => condition,    // size-only rotation
            };
            condition = condition.max_size(max_size.as_bytes());

            // Determine max file count from rotation strategy
            let max_files = match rotation_strategy {
                RotationStrategy::KeepAll => u32::MAX as usize,
                RotationStrategy::KeepOne => 1,
                RotationStrategy::KeepSome(n) => n as usize,
            };

            let log_path = config.log_dir.join(format!("{}.log", config.file_name));
            let file_appender = BasicRollingFileAppender::new(log_path, condition, max_files)
                .map_err(std::io::Error::other)?;

            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            // Wrap with StripAnsiWriter to remove ANSI codes that leak from shared span formatting
            let strip_ansi_writer = StripAnsiWriter::new(non_blocking);

            let layer = fmt::layer()
                .with_timer(make_timer())
                .with_ansi(false)
                .with_file(format_options.file)
                .with_line_number(format_options.line_number)
                .with_thread_ids(format_options.thread_ids)
                .with_thread_names(format_options.thread_names)
                .with_target(format_options.target)
                .with_level(format_options.level)
                .with_writer(strip_ansi_writer);

            (Some(make_layer!(layer, format_options.format)), Some(guard))
        } else {
            // Time-based rotation only using tracing-appender with proper .log extension
            use tracing_appender::rolling::RollingFileAppender;

            let appender_rotation = match rotation {
                Rotation::Daily => tracing_appender::rolling::Rotation::DAILY,
                Rotation::Hourly => tracing_appender::rolling::Rotation::HOURLY,
                Rotation::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
                Rotation::Never => tracing_appender::rolling::Rotation::NEVER,
            };

            let file_appender = RollingFileAppender::builder()
                .rotation(appender_rotation)
                .filename_prefix(&config.file_name)
                .filename_suffix("log")
                .build(&config.log_dir)
                .map_err(std::io::Error::other)?;

            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            // Wrap with StripAnsiWriter to remove ANSI codes that leak from shared span formatting
            let strip_ansi_writer = StripAnsiWriter::new(non_blocking);

            let layer = fmt::layer()
                .with_timer(make_timer())
                .with_ansi(false)
                .with_file(format_options.file)
                .with_line_number(format_options.line_number)
                .with_thread_ids(format_options.thread_ids)
                .with_thread_names(format_options.thread_names)
                .with_target(format_options.target)
                .with_level(format_options.level)
                .with_writer(strip_ansi_writer);

            (Some(make_layer!(layer, format_options.format)), Some(guard))
        }
    } else {
        (None, None)
    };

    // Create custom filter layer if configured
    let custom_filter_layer = custom_filter.map(|f| filter_fn(move |metadata| f(metadata)));

    // Compose the subscriber with all optional layers
    // Note: custom_layer must be added first because it's typed as Layer<Registry>
    let subscriber = Registry::default()
        .with(custom_layer)
        .with(stdout_layer)
        .with(stderr_layer)
        .with(file_layer)
        .with(webview_layer)
        .with(custom_filter_layer)
        .with(filter_with_default);

    tracing::subscriber::set_global_default(subscriber)?;
    tracing::info!("tracing initialized");
    Ok(guard)
}
