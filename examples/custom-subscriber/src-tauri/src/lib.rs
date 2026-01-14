#![allow(clippy::expect_used)] // Standard Tauri app entry point pattern
use tauri::Manager;
use tauri_plugin_tracing::{
    LevelFilter, SpanAwareProfilingExt, SpanTimingLayer, WebviewLayer, init_profiling,
    tracing_appender,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Configure log levels and targets via the builder, then extract the filter.
    let tracing_builder = tauri_plugin_tracing::Builder::default()
        .with_max_level(LevelFilter::DEBUG)
        .with_target("tao::platform_impl", LevelFilter::WARN)
        .with_target("wry", LevelFilter::WARN);

    let filter = tracing_builder.build_filter();

    // Create the span timing layer for CPU profile correlation
    let (span_timing_layer, span_capture) = SpanTimingLayer::new();

    tauri::Builder::default()
        .plugin(tracing_builder.build())
        .plugin(init_profiling()) // Register the CPU profiling plugin
        .invoke_handler(tauri::generate_handler![run_profiled_work])
        .setup(move |app| {
            // Store span capture for profiling correlation
            app.manage(span_capture);

            // Set up file logging
            let log_dir = app.path().app_log_dir()?;
            let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            app.manage(FileLogGuard(guard));

            // Set up subscriber with span timing layer for profiling correlation
            Registry::default()
                .with(span_timing_layer) // Captures span timing for CPU profile correlation
                .with(fmt::layer())
                .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
                .with(WebviewLayer::new(app.handle().clone()))
                .with(filter)
                .init();

            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            tracing::info!(
                log_dir = %log_dir.display(),
                "App initialized with span-aware CPU profiling"
            );
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app, _event| {})
}

/// Example command that runs CPU profiling with span correlation.
#[tauri::command]
fn run_profiled_work(app: tauri::AppHandle) -> Result<String, String> {
    // Start span-aware CPU profiling
    app.start_span_aware_profile()
        .map_err(|e| e.to_string())?;

    // Do some work with tracing spans
    do_work();

    // Stop profiling and get correlation report
    let report = app
        .stop_span_aware_profile()
        .map_err(|e| e.to_string())?;

    Ok(report.to_string())
}

/// Example work function with tracing spans.
#[tracing::instrument]
fn do_work() {
    tracing::info!("Starting work");

    for i in 0..3 {
        process_item(i);
    }

    tracing::info!("Work complete");
}

#[tracing::instrument]
fn process_item(id: u32) {
    tracing::debug!(id, "Processing item");

    // Simulate CPU work
    let mut sum: u64 = 0;
    for j in 0..100_000 {
        sum = sum.wrapping_add(j);
        std::hint::black_box(sum);
    }

    heavy_computation(id);
}

#[tracing::instrument]
fn heavy_computation(id: u32) {
    tracing::trace!(id, "Heavy computation");

    // More CPU work
    let mut result: u64 = 1;
    for k in 1..50_000 {
        result = result.wrapping_mul(k).wrapping_add(1);
        std::hint::black_box(result);
    }
}

/// Wrapper to store the file logging guard in Tauri state.
#[allow(dead_code)]
struct FileLogGuard(WorkerGuard);
