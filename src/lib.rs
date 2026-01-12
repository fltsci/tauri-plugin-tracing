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
//! ```rust,ignore
//! use tauri_plugin_tracing::{Builder, LevelFilter};
//!
//! fn main() {
//!     tauri::Builder::default()
//!         .plugin(
//!             Builder::new()
//!                 .with_max_level(LevelFilter::DEBUG)
//!                 .with_target("my_app", LevelFilter::TRACE)
//!                 .build(),
//!         )
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```
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
use tauri::{AppHandle, Runtime};
use tracing_subscriber::layer::Layered;
use tracing_subscriber::{
    filter::Targets,
    fmt::{Subscriber, SubscriberBuilder},
    prelude::*,
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
use tracing::{Level, instrument};

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
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            builder: SubscriberBuilder::default(),
            log_level: LevelFilter::WARN,
            filter: Targets::default(),
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
    #[cfg(feature = "colored")]
    pub fn with_colors(mut self) -> Self {
        self.builder = self.builder.with_ansi(true);
        self
    }

    /// Initializes the timing state for the application.
    ///
    /// This is called automatically during plugin setup when the `timing`
    /// feature is enabled.
    #[cfg(feature = "timing")]
    pub fn setup_timings<R: Runtime>(&self, app: &AppHandle<R>) {
        use tauri::Manager;
        let timings = Timings::default();
        app.manage(timings);
    }

    fn acquire_logger<R: Runtime>(
        _app_handle: &AppHandle<R>,
        builder: SubscriberBuilder,
        filter: Targets,
        log_level: LevelFilter,
    ) -> Result<Layered<Targets, Subscriber>> {
        Ok(builder.finish().with(filter.with_default(log_level)))
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
        Self::plugin_builder()
            .setup(move |app, _api| {
                #[cfg(feature = "timing")]
                self.setup_timings(app);

                #[cfg(desktop)]
                attach_logger(Self::acquire_logger(
                    app,
                    self.builder,
                    self.filter,
                    self.log_level,
                )?)?;

                Ok(())
            })
            .build()
    }
}

#[instrument(skip(subscriber))]
fn attach_logger(subscriber: Layered<Targets, Subscriber>) -> Result<()> {
    let _ = tracing::subscriber::set_default(subscriber);

    ::tracing::info!("initialized");
    Ok(())
}

fn _rename_file_to_dated() -> Result<()> {
    Err(Error::NotImplemented)
}

fn _get_log_file_path() -> Result<PathBuf> {
    Err(Error::NotImplemented)
}
