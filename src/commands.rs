use tauri::{command, AppHandle, Runtime};
use tracing::event;
use tracing::{debug, error, info, trace, warn, Level};

use crate::models::*;
use crate::Result;

const TARGET: &str = "webview";

#[command]
pub(crate) async fn log<R: Runtime>(
    _app: AppHandle<R>,
    level: LogLevel,
    message: LogMessage,
    call_stack: Option<&str>,
) -> Result<()> {
    let stack = call_stack
        .unwrap_or_default()
        .split("\n")
        .collect::<Vec<&str>>();

    let filtered = stack
        .iter()
        .filter(|line| !line.contains("node_modules"))
        .collect::<Vec<&&str>>()
        .clone();
    let location = match filtered[filtered.len() - 1]
        .split("/")
        .collect::<Vec<&str>>()
        .last()
    {
        Some(location) => location,
        None => ":::",
    };
    Ok(match level {
        LogLevel::Trace => {
            event!(target: TARGET, Level::TRACE, location, %message)
        }
        LogLevel::Debug => {
            event!(target: TARGET, Level::DEBUG, location, %message)
        }
        LogLevel::Info => {
            event!(target: TARGET, Level::INFO, location, %message)
        }
        LogLevel::Warn => {
            event!(target: TARGET, Level::WARN, location, %message)
        }
        LogLevel::Error => {
            for line in stack {
                event!(target: TARGET, Level::TRACE, %line);
            }
            event!(target: TARGET, Level::ERROR, %message)
        }
    })
}
