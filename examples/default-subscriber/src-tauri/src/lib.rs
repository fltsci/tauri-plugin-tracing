use tauri_plugin_tracing::{LevelFilter, Rotation, RotationStrategy};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Configure the tracing plugin with file logging and rotation.
    // Using with_default_subscriber() lets the plugin set up the global subscriber.
    let tracing_plugin = tauri_plugin_tracing::Builder::default()
        .with_colors()
        .with_max_level(LevelFilter::TRACE)
        // Filter out noisy targets
        .with_target("tao::platform_impl", LevelFilter::WARN)
        .with_target("wry", LevelFilter::WARN)
        // Enable file logging to platform log directory
        .with_file_logging()
        // Rotate logs daily and keep the last 7 files
        .with_rotation(Rotation::Daily)
        .with_rotation_strategy(RotationStrategy::KeepSome(7))
        // Let the plugin set up the global tracing subscriber
        .with_default_subscriber()
        .build();

    tauri::Builder::default()
        .plugin(tracing_plugin)
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                app.get_webview_window("main").unwrap().open_devtools();
            }

            tracing::info!("App initialized with file logging and rotation");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app, _event| {})
}
