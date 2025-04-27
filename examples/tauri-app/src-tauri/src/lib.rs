#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // can be configured with similar options as fmt subscriber
        .plugin(
            tauri_plugin_tracing::Builder::default()
                .with_max_level(tauri_plugin_tracing::LevelFilter::TRACE)
                .build(),
        )
        // can also use init() if defaults are ok
        // .plugin(tauri_plugin_tracing::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            use tauri::Manager;
            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();

            ::tauri_plugin_tracing::tracing::info!("App initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
