#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        // .plugin(tauri_plugin_tracing::Builder::new().build())
        .plugin(tauri_plugin_tracing::init())
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
