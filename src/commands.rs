use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::Result;

#[command]
pub(crate) async fn log<R: Runtime>(
    _app: AppHandle<R>,
    level: LogLevel,
    message: LogMessage,
) -> Result<()> {
    Ok(match level {
        LogLevel::Trace => ::tracing::trace!(%message),
        LogLevel::Debug => ::tracing::debug!(%message),
        LogLevel::Info => ::tracing::info!(%message),
        LogLevel::Warn => ::tracing::warn!(%message),
        LogLevel::Error => ::tracing::error!(%message),
    })
}
