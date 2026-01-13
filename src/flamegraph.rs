//! Flamegraph integration for performance profiling.
//!
//! This module provides session-based profiling that records tracing spans
//! and generates flamegraph/flamechart visualizations.
//!
//! # Usage
//!
//! Enable the `flamegraph` feature and use the profiling API:
//!
//! ```javascript
//! import { startProfiling, stopProfiling } from '@fltsci/tauri-plugin-tracing';
//!
//! // Start recording
//! await startProfiling();
//!
//! // ... perform operations to profile ...
//!
//! // Stop recording and generate flamegraph SVG
//! const svgPath = await stopProfiling();
//! ```

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;
use tracing_flame::FlushGuard;

use crate::Result;

/// State for managing flamegraph profiling sessions.
pub struct FlameState {
    /// The flush guard for the current profiling session.
    /// When dropped, the folded stack data is flushed to disk.
    pub(crate) guard: Arc<Mutex<Option<FlushGuard<BufWriter<File>>>>>,
    /// Path to the folded stack output file.
    pub(crate) folded_path: Arc<Mutex<Option<PathBuf>>>,
}

impl Default for FlameState {
    fn default() -> Self {
        Self {
            guard: Arc::new(Mutex::new(None)),
            folded_path: Arc::new(Mutex::new(None)),
        }
    }
}

/// Sets up the flamegraph state for the application.
pub fn setup_flamegraph<R: Runtime>(app: &AppHandle<R>) {
    app.manage(FlameState::default());
}

/// A boxed FlameLayer that can be added to the subscriber.
pub type BoxedFlameLayer =
    Box<dyn tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync + 'static>;

/// Creates a new FlameLayer for the given app handle.
///
/// Returns the layer as a boxed trait object and stores the flush guard in the app state.
/// The folded stack data is written to `{app_log_dir}/profile.folded`.
pub fn create_flame_layer<R: Runtime>(app_handle: &AppHandle<R>) -> Result<BoxedFlameLayer> {
    use tracing_subscriber::Layer;

    let log_dir = app_handle.path().app_log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    let folded_path = log_dir.join("profile.folded");
    let (layer, guard) = tracing_flame::FlameLayer::with_file(&folded_path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Store the guard and path in the app state
    let state = app_handle.state::<FlameState>();
    let guard_lock = state.guard.clone();
    let path_lock = state.folded_path.clone();

    // Use blocking task to store since we're in sync context
    tauri::async_runtime::block_on(async {
        *guard_lock.lock().await = Some(guard);
        *path_lock.lock().await = Some(folded_path);
    });

    Ok(layer.boxed())
}

/// Generates a flamegraph SVG from the folded stack data.
///
/// Returns the path to the generated SVG file.
pub fn generate_flamegraph_svg(folded_path: &std::path::Path) -> Result<PathBuf> {
    use inferno::flamegraph::{self, Options};
    use std::io::{BufRead, BufReader};

    let svg_path = folded_path.with_extension("svg");

    let file = File::open(folded_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    let mut options = Options::default();
    options.title = "Flamegraph".to_string();

    let svg_file = File::create(&svg_path)?;
    flamegraph::from_lines(&mut options, lines.iter().map(|s| s.as_str()), svg_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(svg_path)
}

/// Generates a flamechart SVG from the folded stack data.
///
/// Unlike flamegraphs, flamecharts preserve the exact ordering of events
/// as they were recorded, making it easier to see when each span occurs
/// relative to others.
///
/// Returns the path to the generated SVG file.
pub fn generate_flamechart_svg(folded_path: &std::path::Path) -> Result<PathBuf> {
    use inferno::flamegraph::{self, Options};
    use std::io::{BufRead, BufReader};

    let svg_path = folded_path.with_extension("flamechart.svg");

    let file = File::open(folded_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    let mut options = Options::default();
    options.title = "Flamechart".to_string();
    options.flame_chart = true;

    let svg_file = File::create(&svg_path)?;
    flamegraph::from_lines(&mut options, lines.iter().map(|s| s.as_str()), svg_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(svg_path)
}
