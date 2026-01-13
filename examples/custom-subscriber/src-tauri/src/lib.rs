use tauri::Manager;
use tauri_plugin_tracing::{LevelFilter, WebviewLayer};
use tracing_subscriber::{Registry, layer::SubscriberExt};

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
            // Set up our own subscriber with custom layers.
            // This approach allows adding additional layers like OpenTelemetry,
            // custom formatters, or other tracing integrations.
            let subscriber = Registry::default()
                .with(tracing_subscriber::fmt::layer())
                .with(WebviewLayer::new(app.handle().clone()))
                // Add your custom layers here, e.g.:
                // .with(tracing_opentelemetry::layer())
                .with(filter);

            tracing::subscriber::set_global_default(subscriber)
                .expect("failed to set global subscriber");

            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();

            tracing::info!("App initialized with custom subscriber");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app, _event| {})
}
