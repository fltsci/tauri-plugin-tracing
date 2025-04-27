use tauri::{plugin::TauriPlugin, Manager, Runtime};

pub use models::*;

#[cfg(desktop)]
mod desktop;

mod builder;
mod commands;
mod error;
mod models;

pub use builder::Builder;
pub use error::{Error, Result};
pub use tracing;
pub use tracing::level_filters::LevelFilter;
pub use tracing_appender;
pub use tracing_subscriber;

#[cfg(desktop)]
use desktop::Tracing;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the tracing APIs.
pub trait TracingExt<R: Runtime> {
    fn tracing(&self) -> &Tracing<R>;
}

impl<R: Runtime, T: Manager<R>> crate::TracingExt<R> for T {
    fn tracing(&self) -> &Tracing<R> {
        self.state::<Tracing<R>>().inner()
    }
}

/// Initializes the plugin with default settings.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::Builder::default().build()
}
