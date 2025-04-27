use tauri::{
    Manager, Runtime,
    plugin::{self, TauriPlugin},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::SubscriberBuilder;

use crate::{Error, commands, desktop};

#[derive(Default)]
pub struct Builder(SubscriberBuilder);

impl Builder {
    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("tracing").invoke_handler(tauri::generate_handler![commands::log])
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        Self::plugin_builder()
            .setup(move |app, api| {
                #[cfg(desktop)]
                let plugin = desktop::init(app, api)?;

                app.manage(plugin);
                attach_logger(self)?;
                Ok(())
            })
            .build()
    }

    pub fn with_max_level(mut self, max_level: LevelFilter) -> Self {
        self.0 = self.0.with_max_level(max_level);
        self
    }
}

fn attach_logger(subscriber: Builder) -> Result<(), Error> {
    tracing::subscriber::set_global_default(subscriber.0.finish())?;

    ::tracing::info!("tracing initialized");

    Ok(())
}
