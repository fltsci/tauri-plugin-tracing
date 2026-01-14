//! Flamegraph integration for performance profiling.
//!
//! This module provides session-based profiling that records tracing spans
//! and generates flamegraph/flamechart visualizations.
//!
//! # Usage
//!
//! Enable the `flamegraph` feature and use the profiling API.
//!
//! ## With AppHandle (in Tauri setup)
//!
//! ```rust,ignore
//! let flame_layer = create_flame_layer(app.handle())?;
//! ```
//!
//! ## Early Initialization (before Tauri)
//!
//! ```rust,ignore
//! use tauri_plugin_tracing::{create_flame_layer_with_path, FlameExt};
//!
//! // Create layer before Tauri starts
//! let (flame_layer, flame_guard) = create_flame_layer_with_path(&log_dir.join("profile.folded"))?;
//!
//! // Add to your subscriber
//! registry().with(flame_layer).init();
//!
//! // Later, in Tauri setup, register the guard so JS commands work
//! app.handle().register_flamegraph(flame_guard)?;
//! ```

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};
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

/// A guard that holds the flamegraph flush guard and output path.
///
/// Use this with [`FlameExt::register_flamegraph`] to enable flamegraph generation
/// from the frontend after early initialization.
pub struct FlameGuard {
    guard: FlushGuard<BufWriter<File>>,
    folded_path: PathBuf,
}

/// Creates a new FlameLayer with a custom output path.
///
/// This function does not require a Tauri [`AppHandle`], making it suitable for
/// early initialization before Tauri starts.
///
/// Returns the layer and a [`FlameGuard`] that must be registered with Tauri
/// using [`FlameExt::register_flamegraph`] to enable frontend flamegraph generation.
///
/// # Example
///
/// ```rust,ignore
/// use tauri_plugin_tracing::{create_flame_layer_with_path, FlameExt};
/// use tracing_subscriber::{registry, layer::SubscriberExt, util::SubscriberInitExt};
///
/// let log_dir = std::env::temp_dir().join("my-app");
/// std::fs::create_dir_all(&log_dir)?;
///
/// let (flame_layer, flame_guard) = create_flame_layer_with_path(&log_dir.join("profile.folded"))?;
///
/// registry()
///     .with(tracing_subscriber::fmt::layer())
///     .with(flame_layer)
///     .init();
///
/// // Later, in Tauri setup:
/// // app.handle().register_flamegraph(flame_guard)?;
/// ```
pub fn create_flame_layer_with_path(folded_path: &Path) -> Result<(BoxedFlameLayer, FlameGuard)> {
    use tracing_subscriber::Layer;

    if let Some(parent) = folded_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (layer, guard) = tracing_flame::FlameLayer::with_file(folded_path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let flame_guard = FlameGuard {
        guard,
        folded_path: folded_path.to_path_buf(),
    };

    Ok((layer.boxed(), flame_guard))
}

/// Extension trait for registering flamegraph state with a Tauri application.
///
/// This trait is implemented for [`AppHandle`] and allows registering a [`FlameGuard`]
/// after early initialization, enabling frontend flamegraph generation.
pub trait FlameExt<R: Runtime> {
    /// Registers a [`FlameGuard`] with the application state.
    ///
    /// This must be called in Tauri's setup hook to enable `generateFlamegraph()`
    /// and `generateFlamechart()` from the frontend.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tauri_plugin_tracing::FlameExt;
    ///
    /// tauri::Builder::default()
    ///     .plugin(tauri_plugin_tracing::Builder::new().build())
    ///     .setup(move |app| {
    ///         app.handle().register_flamegraph(flame_guard)?;
    ///         Ok(())
    ///     })
    /// ```
    fn register_flamegraph(&self, guard: FlameGuard) -> Result<()>;
}

impl<R: Runtime> FlameExt<R> for AppHandle<R> {
    fn register_flamegraph(&self, guard: FlameGuard) -> Result<()> {
        let state = self.state::<FlameState>();
        *state
            .guard
            .lock()
            .map_err(|e| crate::Error::LockPoisoned(e.to_string()))? = Some(guard.guard);
        *state
            .folded_path
            .lock()
            .map_err(|e| crate::Error::LockPoisoned(e.to_string()))? = Some(guard.folded_path);
        Ok(())
    }
}

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
    *state
        .guard
        .lock()
        .map_err(|e| crate::Error::LockPoisoned(e.to_string()))? = Some(guard);
    *state
        .folded_path
        .lock()
        .map_err(|e| crate::Error::LockPoisoned(e.to_string()))? = Some(folded_path);

    Ok(layer.boxed())
}

/// Simplifies thread names in folded stack lines.
///
/// Transforms `ThreadId(22)-tokio-runtime-worker;span1;span2 123` into
/// `tokio-worker;span1;span2 123` to consolidate stacks from different worker threads.
fn simplify_thread_names(line: &str) -> String {
    // Find the space before the count (last space in line)
    let (stack, count) = match line.rfind(' ') {
        Some(idx) => (&line[..idx], &line[idx..]),
        None => return line.to_string(),
    };

    // Replace ThreadId(N)-tokio-runtime-worker with just "tokio-worker"
    let simplified = if stack.starts_with("ThreadId(") {
        if let Some(dash_idx) = stack.find(")-") {
            let after_id = &stack[dash_idx + 2..];
            // Consolidate all tokio runtime workers
            if after_id.starts_with("tokio-runtime-worker") {
                if after_id.len() > 20 {
                    format!("tokio-worker{}", &after_id[20..])
                } else {
                    "tokio-worker".to_string()
                }
            } else {
                after_id.to_string()
            }
        } else {
            stack.to_string()
        }
    } else {
        stack.to_string()
    };

    format!("{simplified}{count}")
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
    let lines: Vec<String> = reader
        .lines()
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .map(|line| simplify_thread_names(&line))
        .collect();

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
    let lines: Vec<String> = reader
        .lines()
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .map(|line| simplify_thread_names(&line))
        .collect();

    let mut options = Options::default();
    options.title = "Flamechart".to_string();
    options.flame_chart = true;

    let svg_file = File::create(&svg_path)?;
    flamegraph::from_lines(&mut options, lines.iter().map(|s| s.as_str()), svg_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(svg_path)
}
