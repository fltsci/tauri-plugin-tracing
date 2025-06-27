use tauri::{
    Manager, Runtime,
    plugin::{self, TauriPlugin},
};
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt::SubscriberBuilder,
    prelude::*,
};

use crate::{Error, commands, desktop};

pub struct Builder {
    builder: SubscriberBuilder,
    log_level: LevelFilter,
    filter: Targets,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            builder: SubscriberBuilder::default(),
            log_level: LevelFilter::WARN,
            filter: Targets::default(),
        }
    }
}

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
        self.log_level = max_level;
        self.builder = self.builder.with_max_level(max_level);
        self
    }

    pub fn with_target(mut self, target: &str, level: LevelFilter) -> Self {
        self.filter = self.filter.with_target(target, level);
        self
    }
}

fn attach_logger(builder: Builder) -> Result<(), Error> {
    let subscriber = builder
        .builder
        .finish()
        .with(builder.filter.with_default(builder.log_level));

    tracing::subscriber::set_global_default(subscriber)?;

    #[cfg(debug_assertions)]
    ::tracing::debug!("tracing initialized");

    Ok(())
}
