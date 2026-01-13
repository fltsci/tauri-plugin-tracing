//! Tauri command handlers for the tracing plugin.

use crate::callstack::{CallStack, CallStackLine};
use crate::layer::{LogLevel, LogMessage};
use tauri::Runtime;
use tracing::Level;

#[cfg(feature = "timing")]
use crate::LoggerExt;
#[cfg(feature = "timing")]
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
