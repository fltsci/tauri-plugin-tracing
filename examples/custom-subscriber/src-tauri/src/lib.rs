use tauri::Manager;
use tauri_plugin_tracing::{LevelFilter, WebviewLayer, tracing_appender};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Configure log levels and targets via the builder, then extract the filter.
    // The plugin will NOT set up a global subscriber by default.
    let tracing_builder = tauri_plugin_tracing::Builder::default()
        .with_max_level(LevelFilter::DEBUG)
        // Filter out noisy targets
        .with_target("tao::platform_impl", LevelFilter::WARN)
        .with_target("wry", LevelFilter::WARN);

    // Get the configured filter to use in our custom subscriber
    let filter = tracing_builder.build_filter();

    tauri::Builder::default()
        .plugin(tracing_builder.build())
        .setup(move |app| {
            // Set up file logging using tracing_appender (re-exported by the plugin).
            // This demonstrates file logging with a custom subscriber.
            let log_dir = app.path().app_log_dir()?;
            let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            // Store the guard in Tauri state to keep file logging active.
            // When the guard is dropped, buffered logs are flushed and the worker stops.
            app.manage(FileLogGuard(guard));

            // Set up our own subscriber with custom layers.
            // This approach allows adding additional layers like OpenTelemetry,
            // custom formatters, or other tracing integrations.
            Registry::default()
                .with(fmt::layer()) // stdout
                .with(fmt::layer().with_ansi(false).with_writer(non_blocking)) // file
                .with(WebviewLayer::new(app.handle().clone()))
                // Add your custom layers here, e.g.:
                // .with(tracing_opentelemetry::layer())
                .with(filter)
                .init();

            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            tracing::info!(log_dir = %log_dir.display(), "App initialized with custom subscriber and file logging");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app, _event| {})
}

/// Wrapper to store the file logging guard in Tauri state.
/// The guard must be kept alive for the duration of the application.
#[allow(dead_code)]
struct FileLogGuard(WorkerGuard);
