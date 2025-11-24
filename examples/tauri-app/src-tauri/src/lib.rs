use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use tauri_plugin_tracing::LevelFilter;

#[tracing::instrument]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Filter out unwanted targets
    let targets = Targets::new()
        .with_default(LevelFilter::TRACE)
        .with_target("tao::platform_impl::platform::view", LevelFilter::WARN)
        .with_target(
            "tao::platform_impl::platform::window_delegate",
            LevelFilter::WARN,
        );

    // Configure the app-wide tracing subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(targets)
        .try_init()
        .unwrap();

    // Configure the tracing plugin
    let tracing_plugin = tauri_plugin_tracing::Builder::default()
        .with_colors()
        .with_max_level(LevelFilter::TRACE)
        .build();

    // Add the tracing plugin to the builder
    builder = builder.plugin(tracing_plugin);

    // note: can also use init() if defaults are ok
    // builder = builder.plugin(tauri_plugin_tracing::init());

    builder
        .setup(|app| {
            #[cfg(debug_assertions)]
            use tauri::Manager;
            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();

            ::tauri_plugin_tracing::tracing::info!("App initialized");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app, _event| {})
}
