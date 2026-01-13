//! Tauri command handlers for the tracing plugin.

use crate::callstack::{CallStack, CallStackLine};
use crate::layer::{LogLevel, LogMessage};
use tauri::Runtime;
use tracing::Level;

#[cfg(feature = "timing")]
use crate::LoggerExt;
#[cfg(any(feature = "timing", feature = "flamegraph"))]
use tauri::AppHandle;

#[tauri::command]
#[tracing::instrument(skip_all, fields(w = %CallStackLine::from(webview_window.label())))]
pub fn log<R: Runtime>(
    webview_window: tauri::WebviewWindow<R>,
    level: LogLevel,
    message: LogMessage,
    call_stack: Option<&str>,
) {
    let stack = CallStack::from(call_stack);
    let loc = match level {
        LogLevel::Trace => stack.location(),
        LogLevel::Debug => stack.path(),
        LogLevel::Info => stack.file_name(),
        LogLevel::Warn => stack.path(),
        LogLevel::Error => stack.location(),
    };
    macro_rules! emit_event {
        ($level:expr) => {
            tracing::event!(
                target: "",
                $level,
                %message,
                "" = %loc,
            )
        };
    }
    match level {
        LogLevel::Trace => emit_event!(Level::TRACE),
        LogLevel::Debug => emit_event!(Level::DEBUG),
        LogLevel::Info => emit_event!(Level::INFO),
        LogLevel::Warn => emit_event!(Level::WARN),
        LogLevel::Error => emit_event!(Level::ERROR),
    }
}

#[cfg(feature = "timing")]
#[tauri::command]
pub async fn time<R: Runtime>(app: AppHandle<R>, window: tauri::Window<R>, label: String) {
    use compact_str::ToCompactString;
    // Namespace timer by window label to prevent cross-window interference
    let key = format!("{}:{}", window.label(), label);
    app.time(key.to_compact_string()).await;
}

#[cfg(feature = "timing")]
#[tauri::command]
pub async fn time_end<R: Runtime>(
    app: AppHandle<R>,
    window: tauri::Window<R>,
    label: String,
    call_stack: Option<String>,
) {
    use compact_str::ToCompactString;
    // Namespace timer by window label to prevent cross-window interference
    let key = format!("{}:{}", window.label(), label);
    app.time_end(key.to_compact_string(), call_stack).await;
}

/// Generates a flamegraph SVG from the recorded profiling data.
///
/// Returns the path to the generated SVG file.
#[cfg(feature = "flamegraph")]
#[tauri::command]
pub async fn generate_flamegraph<R: Runtime>(app: AppHandle<R>) -> crate::Result<String> {
    use crate::flamegraph::{FlameState, generate_flamegraph_svg};
    use tauri::Manager;

    let state = app.state::<FlameState>();
    let path_lock = state.folded_path.lock().await;

    let folded_path = path_lock
        .as_ref()
        .ok_or_else(|| crate::Error::Io(std::io::Error::other("No profiling data available")))?;

    // Flush the guard to ensure all data is written
    {
        let mut guard_lock = state.guard.lock().await;
        if let Some(guard) = guard_lock.take() {
            drop(guard);
        }
    }

    let svg_path = generate_flamegraph_svg(folded_path)?;
    Ok(svg_path.to_string_lossy().to_string())
}

/// Generates a flamechart SVG from the recorded profiling data.
///
/// Unlike flamegraphs, flamecharts preserve the exact ordering of events.
/// Returns the path to the generated SVG file.
#[cfg(feature = "flamegraph")]
#[tauri::command]
pub async fn generate_flamechart<R: Runtime>(app: AppHandle<R>) -> crate::Result<String> {
    use crate::flamegraph::{FlameState, generate_flamechart_svg};
    use tauri::Manager;

    let state = app.state::<FlameState>();
    let path_lock = state.folded_path.lock().await;

    let folded_path = path_lock
        .as_ref()
        .ok_or_else(|| crate::Error::Io(std::io::Error::other("No profiling data available")))?;

    // Flush the guard to ensure all data is written
    {
        let mut guard_lock = state.guard.lock().await;
        if let Some(guard) = guard_lock.take() {
            drop(guard);
        }
    }

    let svg_path = generate_flamechart_svg(folded_path)?;
    Ok(svg_path.to_string_lossy().to_string())
}
