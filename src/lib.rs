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

#[cfg(feature = "colored")]
use colored::*;

pub use tracing;
pub use tracing_appender;
pub use tracing_subscriber;

pub use tracing_subscriber::filter::LevelFilter;

use serde::ser::Serializer;
use tracing::{Level, event};

const WEBVIEW_TARGET: &str = "#webview";

#[cfg(target_os = "ios")]
mod ios {
    swift_rs::swift!(pub fn tauri_log(
      level: u8, message: *const std::ffi::c_void
    ));
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Error {
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    TimeFormat(#[from] time::error::Format),
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),
    #[error("Internal logger disabled and cannot be acquired or attached")]
    LoggerNotInitialized,
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),
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

#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    #[cfg(feature = "colored")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dimmed())
    }
    #[cfg(not(feature = "colored"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for CallStackLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl CallStackLine {
    pub fn replace(&self, from: &str, to: &str) -> Self {
        CallStackLine(self.0.replace(from, to))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
        CallStackLine(
            self.0
                .iter()
                .filter(|line| !line.contains("node_modules") && line.contains("src"))
                .map(|line| line.replace("@http://localhost:1420/", "").to_string())
                .collect::<Vec<String>>()
                .clone()
                .join("#"),
        )
    }

    pub fn file_name(&self) -> CallStackLine {
        match self.location().split("/").last() {
            Some(file_name) => CallStackLine(file_name.to_string()),
            None => CallStackLine("unknown".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RecordPayload {
    pub message: String,
    pub level: LogLevel,
}

/// An enum representing the available verbosity levels of the logger.
///
/// It is very similar to the [`log::Level`], but serializes to unsigned ints instead of strings.
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
fn log<R: Runtime>(
    _app: AppHandle<R>,
    level: LogLevel,
    message: LogMessage,
    call_stack: Option<&str>,
) {
    let stack = CallStack::from(call_stack);
    let path = stack.location();
    let file_name = stack.file_name();
    let caller = match level {
        LogLevel::Trace => file_name,
        LogLevel::Debug => file_name,
        LogLevel::Info => file_name,
        LogLevel::Warn => file_name,
        LogLevel::Error => path,
    };
    let span = match level {
        LogLevel::Trace => ::tracing::span!(Level::TRACE, WEBVIEW_TARGET),
        LogLevel::Debug => ::tracing::span!(Level::DEBUG, WEBVIEW_TARGET),
        LogLevel::Info => ::tracing::span!(Level::INFO, WEBVIEW_TARGET),
        LogLevel::Warn => ::tracing::span!(Level::WARN, WEBVIEW_TARGET),
        LogLevel::Error => ::tracing::span!(Level::ERROR, WEBVIEW_TARGET),
    };
    let _enter = span.enter();

    macro_rules! emit_event {
        ($level:expr) => {
            tracing::event!(
                $level,
                ?caller,
                message = %message
            )
        };
    }
    match level {
        LogLevel::Trace => emit_event!(Level::TRACE),
        LogLevel::Debug => emit_event!(Level::DEBUG),
        LogLevel::Info => emit_event!(Level::INFO),
        LogLevel::Warn => emit_event!(Level::WARN),
        LogLevel::Error => {
            for line in &stack.0 {
                event!(Level::ERROR, %line);
            }
            emit_event!(Level::ERROR)
        }
    }
}

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
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_max_level(mut self, max_level: LevelFilter) -> Self {
        self.log_level = max_level;
        self.builder = self.builder.with_max_level(max_level);
        self
    }

    pub fn with_target(mut self, target: &str, level: LevelFilter) -> Self {
        self.filter = self.filter.with_target(target, level);
        self
    }

    #[cfg(feature = "colored")]
    pub fn with_colors(mut self) -> Self {
        self.builder = self.builder.with_ansi(true);
        self
    }

    fn acquire_logger<R: Runtime>(
        _app_handle: &AppHandle<R>,
        builder: SubscriberBuilder,
        filter: Targets,
        log_level: LevelFilter,
    ) -> Result<Layered<Targets, Subscriber>> {
        Ok(builder.finish().with(filter.with_default(log_level)))
    }

    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![log,])
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        Self::plugin_builder()
            .setup(move |app, _api| {
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

fn attach_logger(subscriber: Layered<Targets, Subscriber>) -> Result<()> {
    tracing::subscriber::set_global_default(subscriber)?;

    let span = ::tracing::trace_span!("#attach");
    let _enter = span.enter();
    ::tracing::info!("initialized");

    Ok(())
}

fn _rename_file_to_dated() -> Result<()> {
    Err(Error::NotImplemented)
}

fn _get_log_file_path() -> Result<PathBuf> {
    Err(Error::NotImplemented)
}
