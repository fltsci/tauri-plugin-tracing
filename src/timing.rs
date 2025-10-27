use crate::callstack::*;
use ahash::AHashMap;
use compact_str::{CompactString, ToCompactString};
use std::sync::Mutex;
use std::time::Instant;
use tauri::Manager;
use tauri::Runtime;
use tracing::{Level, event, instrument};

const TIME_END_SPAN: &str = "end";

pub type TimingMap = AHashMap<CompactString, Instant>;

#[derive(Default)]
pub struct Timings(Mutex<TimingMap>);

impl std::ops::Deref for Timings {
    type Target = std::sync::Mutex<TimingMap>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Timings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<R: Runtime, T: Manager<R>> crate::LoggerExt<R> for T {
    fn time(&self, label: CompactString) {
        match self.app_handle().state::<Timings>().lock() {
            Ok(mut timings) => {
                timings.insert(label, std::time::Instant::now());
            }
            Err(e) => {
                event!(Level::ERROR, "Failed to lock timings: {}", e);
            }
        }
    }

    #[instrument(skip_all, name = "time", fields(id = %label))]
    fn time_end(&self, label: CompactString, call_stack: Option<&str>) {
        match self.app_handle().state::<Timings>().lock() {
            Ok(mut timings) => {
                if let Some(started) = timings.remove(&label.to_compact_string()) {
                    event!(
                        target: TIME_END_SPAN,
                        Level::TRACE,
                        message = %format!("{:.3}ms",started.elapsed().as_micros() as f64 / 1000.0),
                         "::" = %CallStack::from(call_stack).file_name()
                    )
                } else {
                    event!(Level::ERROR, "Timing label not found: {}", label);
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Failed to lock timings: {}", e);
            }
        }
    }
}
