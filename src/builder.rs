use tauri::{
    plugin::{self, TauriPlugin},
    Manager, Runtime,
};
use tracing_log::{log::LevelFilter, AsTrace};
use tracing_subscriber::fmt::SubscriberBuilder;

use crate::{commands, desktop, Error};

#[derive(Default)]
pub struct Builder(SubscriberBuilder);

impl std::ops::Deref for Builder {
    type Target = SubscriberBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Builder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
                let builder = self.with_max_level(LevelFilter::Trace);
                attach_logger(builder)?;
                Ok(())
            })
            .build()
    }

    pub fn with_max_level(mut self, max_level: LevelFilter) -> Self {
        self.0 = self.0.with_max_level(max_level.as_trace());
        self
    }
}

fn attach_logger(subscriber: Builder) -> Result<(), Error> {
    tracing::subscriber::set_global_default(subscriber.0.finish())?;

    ::tracing::info!("tracing initialized");

    Ok(())
}

// if !self.is_skip_logger {
// }
// RotationStrategy,
// Target,
// TimezoneStrategy,
// DEFAULT_LOG_TARGETS,
// DEFAULT_MAX_FILE_SIZE,
// DEFAULT_ROTATION_STRATEGY,
// DEFAULT_TIMEZONE_STRATEGY,

// pub struct Builder {
//     rotation_strategy: RotationStrategy,
//     timezone_strategy: TimezoneStrategy,
//     max_file_size: u128,
//     targets: Vec<Target>,
//     is_skip_logger: bool,
// }

// impl Default for Builder {
//     fn default() -> Self {
//         Self {
//             rotation_strategy: DEFAULT_ROTATION_STRATEGY,
//             timezone_strategy: DEFAULT_TIMEZONE_STRATEGY,
//             max_file_size: DEFAULT_MAX_FILE_SIZE,
//             targets: DEFAULT_LOG_TARGETS.into(),
//             is_skip_logger: false,
//         }
//     }
// }
// pub fn rotation_strategy(mut self, rotation_strategy: RotationStrategy) -> Self {
//     // self.rotation_strategy = rotation_strategy;
//     // self.0.with_rotation_strategy(rotation_strategy);
//     rota
//     self
// }

// pub fn timezone_strategy(mut self, timezone_strategy: TimezoneStrategy) -> Self {
//     // self.timezone_strategy = timezone_strategy.clone();

//     // let format =
//     //     time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
//     //         .unwrap();
//     // self.dispatch = self.dispatch.format(move |out, message, record| {
//     //     out.finish(format_args!(
//     //         "{}[{}][{}] {}",
//     //         timezone_strategy.get_now().format(&format).unwrap(),
//     //         record.level(),
//     //         record.target(),
//     //         message
//     //     ))
//     // });
//     self
// }

// pub fn max_file_size(mut self, max_file_size: u128) -> Self {
//     // self.max_file_size = max_file_size;
//     self
// }

// /// Removes all targets. Useful to ignore the default targets and reconfigure them.
// pub fn clear_targets(mut self) -> Self {
//     // self.targets.clear();
//     self
// }

// /// Adds a log target to the logger.
// ///
// /// ```rust
// /// use tauri_plugin_tracing::{Target, TargetKind};
// /// tauri_plugin_tracing::Builder::new()
// ///     .target(Target::new(TargetKind::Webview));
// /// ```
// pub fn target(mut self, target: Target) -> Self {
//     // self.targets.push(target);
//     self.0.;
//     self
// }

// /// Skip the creation and global registration of a logger
// ///
// /// If you wish to use your own global logger, you must call `skip_logger` so that the plugin does not attempt to set a second global logger. In this configuration, no logger will be created and the plugin's `log` command will rely on the result of `log::logger()`. You will be responsible for configuring the logger yourself and any included targets will be ignored. This can also be used with `tracing-log` or if running tests in parallel that require the plugin to be registered.
// /// ```rust
// /// static LOGGER: SimpleLogger = SimpleLogger;
// ///
// /// let subscriber = tracing_subscriber::FmtSubscriber::new();
// /// tracing::subscriber::set_global_default(subscriber)?;
// /// tauri_plugin_tracing::Builder::new()
// ///     .skip_logger();
// /// ```
// pub fn skip_logger(mut self) -> Self {
//     // self.is_skip_logger = true;
//     self
// }

// /// Adds a collection of targets to the logger.
// ///
// /// ```rust
// /// use tauri_plugin_tracing::{Target, TargetKind, WEBVIEW_TARGET};
// /// tauri_plugin_tracing::Builder::new()
// ///     .clear_targets()
// ///     .targets([
// ///         Target::new(TargetKind::Webview),
// ///         Target::new(TargetKind::LogDir { file_name: Some("webview".into()) }).filter(|metadata| metadata.target().starts_with(WEBVIEW_TARGET)),
// ///         Target::new(TargetKind::LogDir { file_name: Some("rust".into()) }).filter(|metadata| !metadata.target().starts_with(WEBVIEW_TARGET)),
// ///     ]);
// /// ```
// pub fn targets(mut self, targets: impl IntoIterator<Item = Target>) -> Self {
//     // self.targets = Vec::from_iter(targets);
//     self
// }

// pub fn format<F>(mut self, formatter: F) -> Self
// where
//     F: Fn(FormatCallback, &Arguments, &Record) + Sync + Send + 'static,
// {
//     // self.dispatch = self.dispatch.format(formatter);
//     self
// }

// pub fn level(mut self, level_filter: impl Into<LevelFilter>) -> Self {
//     // self.dispatch = self.dispatch.level(level_filter.into());
//     self
// }

// pub fn level_for(mut self, module: impl Into<Cow<'static, str>>, level: LevelFilter) -> Self {
//     // self.dispatch = self.dispatch.level_for(module, level);
//     self
// }

// pub fn filter<F>(mut self, filter: F) -> Self
// where
//     F: Fn(&tracing_log::log::Metadata) -> bool + Send + Sync + 'static,
// {
//     // self.dispatch = self.dispatch.filter(filter);
//     self
// }

// #[cfg(feature = "colored")]
// pub fn with_colors(self, colors: fern::colors::ColoredLevelConfig) -> Self {
//     let format =
//         time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
//             .unwrap();

//     let timezone_strategy = self.timezone_strategy.clone();
//     self.format(move |out, message, record| {
//         out.finish(format_args!(
//             "{}[{}][{}] {}",
//             timezone_strategy.get_now().format(&format).unwrap(),
//             colors.color(record.level()),
//             record.target(),
//             message
//         ))
//     })
// }

// fn acquire_logger<R: Runtime>(
//     app_handle: &AppHandle<R>,
//     mut dispatch: fern::Dispatch,
//     rotation_strategy: RotationStrategy,
//     timezone_strategy: TimezoneStrategy,
//     max_file_size: u128,
//     targets: Vec<Target>,
// ) -> Result<
//     (
//         tracing_log::log::LevelFilter,
//         Box<dyn tracing_log::log::Log>,
//     ),
//     Error,
// > {
//     let app_name = &app_handle.package_info().name;

//     // setup targets
//     for target in targets {
//         let mut target_dispatch = fern::Dispatch::new();
//         for filter in target.filters {
//             target_dispatch = target_dispatch.filter(filter);
//         }

//         let logger = match target.kind {
//             #[cfg(desktop)]
//             TargetKind::Stdout => std::io::stdout().into(),
//             #[cfg(desktop)]
//             TargetKind::Stderr => std::io::stderr().into(),
//             TargetKind::Folder { path, file_name } => {
//                 if !path.exists() {
//                     fs::create_dir_all(&path)?;
//                 }

//                 fern::log_file(get_log_file_path(
//                     &path,
//                     file_name.as_deref().unwrap_or(app_name),
//                     &rotation_strategy,
//                     &timezone_strategy,
//                     max_file_size,
//                 )?)?
//                 .into()
//             }
//             #[cfg(desktop)]
//             TargetKind::LogDir { file_name } => {
//                 let path = app_handle.path().app_log_dir()?;
//                 if !path.exists() {
//                     fs::create_dir_all(&path)?;
//                 }

//                 fern::log_file(get_log_file_path(
//                     &path,
//                     file_name.as_deref().unwrap_or(app_name),
//                     &rotation_strategy,
//                     &timezone_strategy,
//                     max_file_size,
//                 )?)?
//                 .into()
//             }
//             TargetKind::Webview => {
//                 let app_handle = app_handle.clone();

//                 fern::Output::call(move |record| {
//                     let payload = RecordPayload {
//                         message: record.args().to_string(),
//                         level: record.level().into(),
//                     };
//                     let app_handle = app_handle.clone();
//                     tauri::async_runtime::spawn(async move {
//                         let _ = app_handle.emit("tracing://log", payload);
//                     });
//                 })
//             }
//         };
//         target_dispatch = target_dispatch.chain(logger);

//         dispatch = dispatch.chain(target_dispatch);
//     }

//     Ok(dispatch.into_log())
// }

// #[allow(clippy::type_complexity)]
// pub fn split<R: Runtime>(
//     self,
//     app_handle: &AppHandle<R>,
// ) -> Result<
//     (
//         TauriPlugin<R>,
//         tracing_log::log::LevelFilter,
//         Box<dyn tracing_log::log::Log>,
//     ),
//     Error,
// > {
//     if self.is_skip_logger {
//         return Err(Error::LoggerNotInitialized);
//     }
//     let plugin = Self::plugin_builder();
//     let (max_level, log) = Self::acquire_logger(
//         app_handle,
//         self.dispatch,
//         self.rotation_strategy,
//         self.timezone_strategy,
//         self.max_file_size,
//         self.targets,
//     )?;

//     Ok((plugin.build(), max_level, log))
// }
