use crate::callstack::*;
use ahash::{HashMap, HashMapExt};
use compact_str::CompactString;
use std::time::Instant;
use tauri::Manager;
use tauri::Runtime;
use tokio::sync::Mutex;
use tracing::{Level, event, instrument};

const TIME_END_SPAN: &str = "end";

pub type TimingMap = HashMap<CompactString, Instant>;

pub struct Timings(Mutex<TimingMap>);

impl Default for Timings {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

impl std::ops::Deref for Timings {
    type Target = Mutex<TimingMap>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Timings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl<R: Runtime, T: Manager<R> + std::marker::Sync> crate::LoggerExt<R> for T {
    async fn time(&self, label: CompactString) {
        self.app_handle()
            .state::<Timings>()
            .lock()
            .await
            .insert(label, std::time::Instant::now());
    }

    #[instrument(skip_all, name = "time", fields(id = %label))]
    async fn time_end(&self, label: CompactString, call_stack: Option<String>) {
        let caller = CallStack::from(call_stack).file_name();
        if let Some(started) = self
            .app_handle()
            .state::<Timings>()
            .lock()
            .await
            .remove(&label)
        {
            event!(
                target: TIME_END_SPAN,
                Level::TRACE,
                message = %format!("{:.3}ms",started.elapsed().as_micros() as f64 / 1000.0),
                 "::" = %caller
            )
        } else {
            event!(
                target: TIME_END_SPAN,
                Level::WARN,
                message = %format!("Timing label not found: {}", label),
                "::" = %caller
            );
        }
    }
}
