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
//! ```rust,ignore
//! use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
//! use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
//!
//! fn main() {
//!     let tracing_builder = Builder::new()
//!         .with_max_level(LevelFilter::DEBUG)
//!         .with_target("hyper", LevelFilter::WARN);
//!     let filter = tracing_builder.build_filter();
//!
//!     tauri::Builder::default()
//!         .plugin(tracing_builder.build())
//!         .setup(move |app| {
//!             let subscriber = Registry::default()
//!                 .with(fmt::layer())
//!                 .with(WebviewLayer::new(app.handle().clone()))
//!                 .with(filter);
//!             tracing::subscriber::set_global_default(subscriber)?;
//!             Ok(())
//!         })
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```
//!
//! ## Quick Start
//!
//! For simple applications, use [`Builder::with_default_subscriber()`] to let
//! the plugin handle all tracing setup:
//!
//! ```rust,ignore
//! use tauri_plugin_tracing::{Builder, LevelFilter};
//!
//! tauri::Builder::default()
//!     .plugin(
//!         Builder::new()
//!             .with_max_level(LevelFilter::DEBUG)
//!             .with_default_subscriber()  // Let plugin set up tracing
//!             .build(),
//!     )
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```
//!
//! ## File Logging
//!
//! File logging requires [`Builder::with_default_subscriber()`]:
//!
//! ```rust,ignore
//! Builder::new()
//!     .with_max_level(LevelFilter::DEBUG)
//!     .with_file_logging()
//!     .with_default_subscriber()
//!     .build()
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
    filter::Targets,
    fmt::{self, SubscriberBuilder},
    layer::SubscriberExt,
};

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
/// ```rust,ignore
/// use tauri_plugin_tracing::LoggerExt;
///
/// // In a Tauri command:
/// app.time("my_operation".into()).await;
/// // ... perform operation ...
/// app.time_end("my_operation".into(), None).await;
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
/// ```rust,ignore
/// use tauri_plugin_tracing::{Builder, Target};
/// use std::path::PathBuf;
///
/// // Log to stdout and webview (default behavior)
/// Builder::new()
///     .targets([Target::Stdout, Target::Webview])
///     .build();
///
/// // Log to file and webview only (no console)
/// Builder::new()
///     .targets([
///         Target::LogDir { file_name: None },
///         Target::Webview,
///     ])
///     .build();
///
/// // Log to stderr instead of stdout
/// Builder::new()
///     .clear_targets()
///     .target(Target::Stderr)
///     .target(Target::Webview)
///     .build();
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
/// ```rust,ignore
/// use tauri_plugin_tracing::{Builder, Rotation};
///
/// Builder::new()
///     .with_file_logging()
///     .with_rotation(Rotation::Daily)  // Create new file each day
///     .build()
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
/// ```rust,ignore
/// use tauri_plugin_tracing::{Builder, RotationStrategy};
///
/// Builder::new()
///     .with_file_logging()
///     .with_rotation_strategy(RotationStrategy::KeepSome(7))  // Keep 7 most recent files
///     .build()
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

/// Stores the WorkerGuard to ensure logs are flushed on shutdown.
/// This must be kept alive for the lifetime of the application.
struct LogGuard(#[allow(dead_code)] Option<WorkerGuard>);

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
/// ```rust,ignore
/// use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
/// use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
///
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
///     })
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
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
/// ```rust,ignore
/// use tauri_plugin_tracing::{Builder, LevelFilter};
///
/// let plugin = Builder::new()
///     .with_max_level(LevelFilter::DEBUG)
///     .with_target("hyper", LevelFilter::WARN)  // Reduce noise from hyper
///     .with_target("my_app", LevelFilter::TRACE)  // Verbose logging for your app
///     .build();
/// ```
pub struct Builder {
    builder: SubscriberBuilder,
    log_level: LevelFilter,
    filter: Targets,
    targets: Vec<Target>,
    rotation: Rotation,
    rotation_strategy: RotationStrategy,
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
            targets: vec![Target::Stdout, Target::Webview],
            rotation: Rotation::default(),
            rotation_strategy: RotationStrategy::default(),
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
    /// ```rust,ignore
    /// Builder::new().with_max_level(LevelFilter::DEBUG)
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
    /// ```rust,ignore
    /// Builder::new()
    ///     .with_max_level(LevelFilter::INFO)
    ///     .with_target("my_app::database", LevelFilter::DEBUG)
    ///     .with_target("hyper", LevelFilter::WARN)
    /// ```
    pub fn with_target(mut self, target: &str, level: LevelFilter) -> Self {
        self.filter = self.filter.with_target(target, level);
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
    /// ```rust,ignore
    /// Builder::new()
    ///     .with_max_level(LevelFilter::DEBUG)
    ///     .with_file_logging()
    ///     .build()
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
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, Rotation};
    ///
    /// Builder::new()
    ///     .with_file_logging()
    ///     .with_rotation(Rotation::Hourly)  // Rotate every hour
    ///     .build()
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
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, RotationStrategy};
    ///
    /// Builder::new()
    ///     .with_file_logging()
    ///     .with_rotation_strategy(RotationStrategy::KeepSome(7))  // Keep 7 files
    ///     .build()
    /// ```
    pub fn with_rotation_strategy(mut self, strategy: RotationStrategy) -> Self {
        self.rotation_strategy = strategy;
        self
    }

    /// Adds a log output target.
    ///
    /// By default, logs are sent to [`Target::Stdout`] and [`Target::Webview`].
    /// Use this method to add additional targets.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// // Add file logging to the default targets
    /// Builder::new()
    ///     .target(Target::LogDir { file_name: None })
    ///     .build();
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
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// // Log only to file and webview (no stdout)
    /// Builder::new()
    ///     .targets([
    ///         Target::LogDir { file_name: None },
    ///         Target::Webview,
    ///     ])
    ///     .build();
    ///
    /// // Log only to stderr
    /// Builder::new()
    ///     .targets([Target::Stderr])
    ///     .build();
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
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, Target};
    ///
    /// // Start fresh and only log to webview
    /// Builder::new()
    ///     .clear_targets()
    ///     .target(Target::Webview)
    ///     .build();
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
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, LevelFilter};
    ///
    /// // Let the plugin set up everything
    /// tauri::Builder::default()
    ///     .plugin(
    ///         Builder::new()
    ///             .with_max_level(LevelFilter::DEBUG)
    ///             .with_file_logging()
    ///             .with_default_subscriber()  // Opt-in to global subscriber
    ///             .build()
    ///     )
    ///     .run(tauri::generate_context!())
    ///     .expect("error while running tauri application");
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
    /// ```rust,ignore
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

    /// Returns the configured filter based on log level and per-target settings.
    ///
    /// Use this when setting up your own subscriber to apply the same filtering
    /// configured via [`with_max_level()`](Self::with_max_level) and
    /// [`with_target()`](Self::with_target).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
    /// use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
    ///
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
    ///     })
    ///     .run(tauri::generate_context!())
    ///     .expect("error while running tauri application");
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
    /// ```rust,ignore
    /// tauri::Builder::default()
    ///     .plugin(Builder::new().build())
    ///     .run(tauri::generate_context!())
    ///     .expect("error while running tauri application");
    /// ```
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let log_level = self.log_level;
        let filter = self.filter;
        let targets = self.targets;
        let rotation = self.rotation;
        let rotation_strategy = self.rotation_strategy;
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
                        &targets,
                        rotation,
                        rotation_strategy,
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
fn acquire_logger<R: Runtime>(
    app_handle: &AppHandle<R>,
    log_level: LevelFilter,
    filter: Targets,
    targets: &[Target],
    rotation: Rotation,
    rotation_strategy: RotationStrategy,
    #[cfg(feature = "colored")] use_colors: bool,
) -> Result<Option<WorkerGuard>> {
    use std::io;

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

    // Determine if ANSI should be enabled.
    // When file logging is enabled, disable ANSI on all layers because
    // tracing-subscriber shares span field formatting between layers.
    #[cfg(feature = "colored")]
    let use_ansi = use_colors && file_config.is_none();
    #[cfg(not(feature = "colored"))]
    let use_ansi = false;

    // Create optional layers based on targets
    let stdout_layer = if has_stdout {
        Some(fmt::layer().with_ansi(use_ansi).with_target(true))
    } else {
        None
    };

    let stderr_layer = if has_stderr {
        Some(
            fmt::layer()
                .with_ansi(use_ansi)
                .with_target(true)
                .with_writer(io::stderr),
        )
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
        cleanup_old_logs(&config.log_dir, &config.file_name, rotation_strategy)?;

        let file_appender = match rotation {
            Rotation::Daily => tracing_appender::rolling::daily(&config.log_dir, &config.file_name),
            Rotation::Hourly => {
                tracing_appender::rolling::hourly(&config.log_dir, &config.file_name)
            }
            Rotation::Minutely => {
                tracing_appender::rolling::minutely(&config.log_dir, &config.file_name)
            }
            Rotation::Never => tracing_appender::rolling::never(&config.log_dir, &config.file_name),
        };
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let layer = fmt::layer()
            .with_ansi(false)
            .with_target(true)
            .with_writer(non_blocking);

        (Some(layer), Some(guard))
    } else {
        (None, None)
    };

    // Compose the subscriber with all optional layers
    let subscriber = Registry::default()
        .with(stdout_layer)
        .with(stderr_layer)
        .with(file_layer)
        .with(webview_layer)
        .with(filter_with_default);

    tracing::subscriber::set_global_default(subscriber)?;
    tracing::info!("tracing initialized");
    Ok(guard)
}
