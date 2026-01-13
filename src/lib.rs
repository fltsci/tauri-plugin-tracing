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
//! For simple file logging, use [`Builder::with_file_logging()`]:
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
//! For custom subscribers, use [`tracing_appender`] directly (re-exported by this crate):
//!
//! ```rust,no_run
//! # use tauri::Manager;
//! # use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter, tracing_appender};
//! # use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};
//! let tracing_builder = Builder::new().with_max_level(LevelFilter::DEBUG);
//! let filter = tracing_builder.build_filter();
//!
//! tauri::Builder::default()
//!     .plugin(tracing_builder.build())
//!     .setup(move |app| {
//!         let log_dir = app.path().app_log_dir()?;
//!         let file_appender = tracing_appender::rolling::daily(&log_dir, "app");
//!         let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
//!         // Store _guard in Tauri state to keep file logging active
//!
//!         let subscriber = Registry::default()
//!             .with(fmt::layer())
//!             .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
//!             .with(WebviewLayer::new(app.handle().clone()))
//!             .with(filter);
//!         tracing::subscriber::set_global_default(subscriber)?;
//!         Ok(())
//!     });
//!     // .run(tauri::generate_context!())
//! ```
//!
//! Log files rotate daily and are written to:
//! - **macOS**: `~/Library/Logs/{bundle_identifier}/app.YYYY-MM-DD.log`
//! - **Linux**: `~/.local/share/{bundle_identifier}/logs/app.YYYY-MM-DD.log`
//! - **Windows**: `%LOCALAPPDATA%/{bundle_identifier}/logs/app.YYYY-MM-DD.log`
//!
//! ## Early Initialization
//!
//! For maximum control, initialize tracing before creating the Tauri app. This
//! pattern uses [`tracing_subscriber::registry()`] with [`init()`](tracing_subscriber::util::SubscriberInitExt::init)
//! and passes a minimal [`Builder`] to the plugin:
//!
//! ```rust,no_run
//! use tauri_plugin_tracing::{Builder, StripAnsiWriter, tracing_appender};
//! use tracing::Level;
//! use tracing_subscriber::filter::Targets;
//! use tracing_subscriber::layer::SubscriberExt;
//! use tracing_subscriber::util::SubscriberInitExt;
//! use tracing_subscriber::{fmt, registry};
//!
//! fn setup_logger() -> Builder {
//!     let log_dir = std::env::temp_dir().join("my-app");
//!     let _ = std::fs::create_dir_all(&log_dir);
//!
//!     let file_appender = tracing_appender::rolling::daily(&log_dir, "app");
//!     let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
//!     std::mem::forget(guard); // Keep file logging active for app lifetime
//!
//!     let targets = Targets::new()
//!         .with_default(Level::DEBUG)
//!         .with_target("hyper", Level::WARN)
//!         .with_target("reqwest", Level::WARN);
//!
//!     registry()
//!         .with(fmt::layer().with_ansi(true))
//!         .with(fmt::layer().with_writer(StripAnsiWriter::new(non_blocking)).with_ansi(false))
//!         .with(targets)
//!         .init();
//!
//!     // Return minimal builder - logging is already configured
//!     Builder::new()
//! }
//!
//! fn main() {
//!     let builder = setup_logger();
//!     tauri::Builder::default()
//!         .plugin(builder.build());
//!         // .run(tauri::generate_context!())
//! }
//! ```
//!
//! This approach is useful when you need logging available before Tauri starts,
//! or when you want full control over the subscriber configuration.
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
mod commands;
mod error;
#[cfg(feature = "flamegraph")]
mod flamegraph;
mod layer;
mod strip_ansi;
#[cfg(feature = "timing")]
mod timing;
mod types;

use std::path::PathBuf;
use tauri::plugin::{self, TauriPlugin};
use tauri::{AppHandle, Manager, Runtime};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    Layer as _, Registry,
    filter::{Targets, filter_fn},
    fmt::{self, SubscriberBuilder},
    layer::SubscriberExt,
};

// Re-export public types from modules
pub use callstack::{CallStack, CallStackLine};
pub use commands::log;
#[cfg(feature = "timing")]
pub use commands::{time, time_end};
pub use error::{Error, Result};
pub use layer::{LogLevel, LogMessage, RecordPayload, WebviewLayer};
pub use strip_ansi::{StripAnsiWriter, StripAnsiWriterGuard};
#[cfg(feature = "timing")]
pub use timing::{TimingMap, Timings};
pub use types::{
    FormatOptions, LogFormat, MaxFileSize, Rotation, RotationStrategy, Target, TimezoneStrategy,
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
pub type BoxedLayer = Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static>;

#[cfg(feature = "flamegraph")]
pub use flamegraph::*;

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

#[cfg(target_os = "ios")]
mod ios {
    swift_rs::swift!(pub fn tauri_log(
      level: u8, message: *const std::ffi::c_void
    ));
}

/// Stores the WorkerGuard to ensure logs are flushed on shutdown.
/// This must be kept alive for the lifetime of the application.
struct LogGuard(#[allow(dead_code)] Option<WorkerGuard>);

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
    #[cfg(feature = "flamegraph")]
    enable_flamegraph: bool,
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
            #[cfg(feature = "flamegraph")]
            enable_flamegraph: false,
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

    /// Enables flamegraph profiling.
    ///
    /// When enabled, tracing spans are recorded to a folded stack format file
    /// that can be converted to a flamegraph or flamechart visualization.
    ///
    /// The folded stack data is written to `{app_log_dir}/profile.folded`.
    /// Use the `generate_flamegraph` or `generate_flamechart` commands to
    /// convert this data to an SVG visualization.
    ///
    /// Only applies when using [`with_default_subscriber()`](Self::with_default_subscriber).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tauri_plugin_tracing::Builder;
    ///
    /// Builder::new()
    ///     .with_flamegraph()
    ///     .with_default_subscriber()
    ///     .build()
    /// ```
    #[cfg(feature = "flamegraph")]
    pub fn with_flamegraph(mut self) -> Self {
        self.enable_flamegraph = true;
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

    #[cfg(all(feature = "timing", feature = "flamegraph"))]
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![
            commands::log,
            commands::time,
            commands::time_end,
            commands::generate_flamegraph,
            commands::generate_flamechart
        ])
    }

    #[cfg(all(feature = "timing", not(feature = "flamegraph")))]
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![
            commands::log,
            commands::time,
            commands::time_end
        ])
    }

    #[cfg(all(not(feature = "timing"), feature = "flamegraph"))]
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![
            commands::log,
            commands::generate_flamegraph,
            commands::generate_flamechart
        ])
    }

    #[cfg(all(not(feature = "timing"), not(feature = "flamegraph")))]
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![commands::log,])
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

        #[cfg(feature = "flamegraph")]
        let enable_flamegraph = self.enable_flamegraph;

        Self::plugin_builder()
            .setup(move |app, _api| {
                #[cfg(feature = "timing")]
                setup_timings(app);

                #[cfg(feature = "flamegraph")]
                setup_flamegraph(app);

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
                        #[cfg(feature = "flamegraph")]
                        enable_flamegraph,
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
    let timings = timing::Timings::default();
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
    #[cfg(feature = "flamegraph")] enable_flamegraph: bool,
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
            condition = condition.max_size(max_size.0);

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

    // Create flame layer if flamegraph feature is enabled
    #[cfg(feature = "flamegraph")]
    let flame_layer = if enable_flamegraph {
        Some(create_flame_layer(app_handle)?)
    } else {
        None
    };

    // Create custom filter layer if configured
    let custom_filter_layer = custom_filter.map(|f| filter_fn(move |metadata| f(metadata)));

    // Compose the subscriber with all optional layers
    // Note: Boxed layers (custom_layer, flame_layer) must be combined and added first
    // because they're typed as Layer<Registry> and the subscriber type changes after each .with()
    #[cfg(feature = "flamegraph")]
    let combined_boxed_layer: Option<BoxedLayer> = match (custom_layer, flame_layer) {
        (Some(c), Some(f)) => {
            use tracing_subscriber::Layer;
            Some(c.and_then(f).boxed())
        }
        (Some(c), None) => Some(c),
        (None, Some(f)) => Some(f),
        (None, None) => None,
    };

    #[cfg(not(feature = "flamegraph"))]
    let combined_boxed_layer = custom_layer;

    let subscriber = Registry::default()
        .with(combined_boxed_layer)
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
