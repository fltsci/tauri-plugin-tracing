//! WebviewLayer for forwarding log events to the frontend.

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::{AppHandle, Emitter, Runtime};
use tracing_subscriber::Layer;

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
/// # use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt, fmt};
/// let builder = Builder::new()
///     .with_max_level(LevelFilter::DEBUG);
/// let filter = builder.build_filter();
///
/// tauri::Builder::default()
///     .plugin(builder.build())
///     .setup(move |app| {
///         Registry::default()
///             .with(fmt::layer())
///             .with(WebviewLayer::new(app.handle().clone()))
///             .with(filter)
///             .init();
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
