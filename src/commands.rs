use tauri::{AppHandle, Runtime, command};
use tracing::{Level, event};

use crate::Result;
use crate::models::*;

const TARGET: &str = "webview";

#[command]
pub(crate) async fn log<R: Runtime>(
    _app: AppHandle<R>,
    level: LogLevel,
    message: LogMessage,
    call_stack: Option<&str>,
) -> Result<()> {
    let stack = CallStack::from(call_stack);
    let location = stack.location();
    match level {
        LogLevel::Trace => {
            event!(target: TARGET, Level::TRACE, %location, %message)
        }
        LogLevel::Debug => {
            event!(target: TARGET, Level::DEBUG, %location, %message)
        }
        LogLevel::Info => {
            event!(target: TARGET, Level::INFO, %location, %message)
        }
        LogLevel::Warn => {
            event!(target: TARGET, Level::WARN, %location, %message)
        }
        LogLevel::Error => {
            for line in &stack.0 {
                event!(target: TARGET, Level::TRACE, %line);
            }
            event!(target: TARGET, Level::ERROR, %location, %message)
        }
    };
    Ok(())
}
