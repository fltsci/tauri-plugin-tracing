use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

/// Access to the tracing APIs.
pub struct Tracing<R: Runtime>(AppHandle<R>);

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Tracing<R>> {
    Ok(Tracing(app.clone()))
}

// use crate::models::*;

// impl<R: Runtime> Tracing<R> {
//     pub fn clear(&self) {
//         print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
//     }

//     pub fn debug(&self, message: LogMessage) {
//         ::tracing::debug!(?message);
//     }

//     pub fn error(&self, message: LogMessage) {
//         ::tracing::error!(?message);
//     }

//     pub fn log(&self, message: LogMessage) {
//         ::tracing::info!(?message);
//     }

//     pub fn trace(&self, message: LogMessage) {
//         ::tracing::trace!(?message);
//     }

//     pub fn warn(&self, message: LogMessage) {
//         ::tracing::warn!(?message);
//     }
// }

// let level = tracing_log::log::Level::from(LogLevel::Trace);

// let target = if let Some(location) = location {
//     format!("{WEBVIEW_TARGET}:{location}")
// } else {
//     WEBVIEW_TARGET.to_string()
// };

// let mut builder = RecordBuilder::new();
// builder.level(level).target(&target).file(file).line(line);

// let key_values = key_values.unwrap_or_default();
// let mut kv = HashMap::new();
// for (k, v) in key_values.iter() {
//     kv.insert(k.as_str(), v.as_str());
// }
// builder.key_values(&kv);

// pub fn assert(&self, condition: bool, message: LogMessage) {
//     assert!(condition, "assert failed: {:?}", message);
// }

// pub fn count(&self, message: LogMessage) {
//     todo!()
// }

// pub fn count_reset(&self, message: LogMessage) {
//     todo!()
// }

// pub fn dir(&self, message: LogMessage) {
//     todo!()
// }

// pub fn dirxml(&self, message: LogMessage) {
//     todo!()
// }
// pub fn group(&self, message: LogMessage) {
//     todo!()
// }

// pub fn group_collapsed(&self, message: LogMessage) {
//     todo!()
// }

// pub fn group_end(&self) {
//     todo!()
// }

// pub fn profile(&self, profile_name: &str) {
//     ::tracing::span!(::tracing::Level::INFO, "profile", name = profile_name)
//         .in_scope(|| todo!());
// }

// pub fn profile_end(&self) {
//     todo!()
// }

// pub fn table(&self, data: Vec<&str>, columns: Vec<Vec<&str>>) {
//     todo!()
// }

// pub fn time(&self, label: &str) {
//     todo!()
// }

// pub fn time_end(&self, label: &str) {
//     todo!()
// }

// pub fn time_log(&self, label: &str, message: LogMessage) {
//     todo!()
// }
