//! CPU profiling integration via `tauri-plugin-profiling`.
//!
//! This module provides tracing-aware CPU profiling that automatically creates
//! spans and logs profiling events, with optional span timing correlation.
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use tauri_plugin_tracing::{Builder, LevelFilter, TracedProfilingExt, init_profiling};
//!
//! tauri::Builder::default()
//!     .plugin(Builder::new().with_max_level(LevelFilter::DEBUG).build())
//!     .plugin(init_profiling())
//!     .setup(|app| {
//!         app.start_cpu_profile_traced()?;
//!         // ... do work ...
//!         let result = app.stop_cpu_profile_traced()?;
//!         Ok(())
//!     })
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```
//!
//! ## Span-Correlated Profiling
//!
//! Use [`SpanTimingLayer`] to correlate CPU samples with active tracing spans:
//!
//! ```rust,no_run
//! use tauri_plugin_tracing::{
//!     Builder, LevelFilter, SpanTimingLayer, SpanAwareProfilingExt, init_profiling
//! };
//! use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};
//!
//! // Create the span timing layer
//! let (span_layer, span_capture) = SpanTimingLayer::new();
//!
//! tauri::Builder::default()
//!     .plugin(Builder::new().build())
//!     .plugin(init_profiling())
//!     .setup(move |app| {
//!         // Register the span capture for later correlation
//!         app.manage(span_capture);
//!
//!         // Initialize subscriber with span timing layer
//!         Registry::default()
//!             .with(span_layer)
//!             .with(tracing_subscriber::fmt::layer())
//!             .init();
//!
//!         // Profile with span correlation
//!         app.start_span_aware_profile()?;
//!         // ... do work with tracing spans ...
//!         let report = app.stop_span_aware_profile()?;
//!         println!("{}", report);
//!         Ok(())
//!     })
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_profiling::ProfilingExt;
use tracing::span::Attributes;
use tracing::{Id, Span, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

// Re-export all types from tauri-plugin-profiling
pub use tauri_plugin_profiling::{
    Error as ProfilingError, ProfileResult, ProfilingConfig, ProfilingExt as ProfilingExtBase,
    Result as ProfilingResult, StartOptions, init as init_profiling,
    init_with_config as init_profiling_with_config,
};

// ============================================================================
// Span Timing Layer
// ============================================================================

/// A recorded span timing event.
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Span name (target::name format)
    pub name: String,
    /// Span ID
    pub span_id: u64,
    /// Parent span ID (0 if none)
    pub parent_id: u64,
    /// Event type
    pub event_type: SpanEventType,
    /// Timestamp relative to capture start (microseconds)
    pub timestamp_us: u64,
    /// Thread ID where the event occurred
    pub thread_id: u64,
}

/// Type of span event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanEventType {
    /// Span was entered
    Enter,
    /// Span was exited
    Exit,
    /// Span was closed (dropped)
    Close,
}

/// Shared state for span timing capture.
#[derive(Debug)]
pub struct SpanTimingCapture {
    events: Mutex<Vec<SpanEvent>>,
    start_time: Mutex<Option<Instant>>,
    capturing: AtomicBool,
    next_id: AtomicU64,
}

impl SpanTimingCapture {
    /// Creates a new span timing capture.
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            start_time: Mutex::new(None),
            capturing: AtomicBool::new(false),
            next_id: AtomicU64::new(1),
        }
    }

    /// Starts capturing span timing events.
    pub fn start_capture(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
        if let Ok(mut start) = self.start_time.lock() {
            *start = Some(Instant::now());
        }
        self.capturing.store(true, Ordering::SeqCst);
    }

    /// Stops capturing and returns all recorded events.
    pub fn stop_capture(&self) -> Vec<SpanEvent> {
        self.capturing.store(false, Ordering::SeqCst);
        if let Ok(mut events) = self.events.lock() {
            std::mem::take(&mut *events)
        } else {
            Vec::new()
        }
    }

    /// Returns whether capture is currently active.
    pub fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    fn record_event(&self, name: String, span_id: u64, parent_id: u64, event_type: SpanEventType) {
        if !self.capturing.load(Ordering::SeqCst) {
            return;
        }

        let timestamp_us = if let Ok(start) = self.start_time.lock() {
            start.map(|s| s.elapsed().as_micros() as u64).unwrap_or(0)
        } else {
            0
        };

        // Use hash of thread ID since as_u64() is unstable
        let thread_id = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::thread::current().id().hash(&mut hasher);
            hasher.finish()
        };

        let event = SpanEvent {
            name,
            span_id,
            parent_id,
            event_type,
            timestamp_us,
            thread_id,
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

/// A tracing layer that captures span timing for correlation with CPU profiles.
///
/// Create with [`SpanTimingLayer::new()`], which returns both the layer and
/// a [`SpanTimingCapture`] handle for controlling capture and retrieving events.
pub struct SpanTimingLayer {
    capture: Arc<SpanTimingCapture>,
}

impl SpanTimingLayer {
    /// Creates a new span timing layer and its capture handle.
    ///
    /// The returned [`SpanTimingCapture`] should be stored (e.g., in Tauri state)
    /// and used to start/stop capture and retrieve events.
    pub fn new() -> (Self, Arc<SpanTimingCapture>) {
        let capture = Arc::new(SpanTimingCapture::new());
        (
            Self {
                capture: capture.clone(),
            },
            capture,
        )
    }
}

impl Default for SpanTimingLayer {
    fn default() -> Self {
        Self::new().0
    }
}

// Store our internal ID on spans
struct SpanTimingId(u64);

impl<S> Layer<S> for SpanTimingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let internal_id = self.capture.next_id();

        // Store our internal ID on the span
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanTimingId(internal_id));
        }

        // Get parent ID
        let parent_id = attrs
            .parent()
            .and_then(|pid| ctx.span(pid))
            .and_then(|span| span.extensions().get::<SpanTimingId>().map(|id| id.0))
            .or_else(|| {
                ctx.lookup_current()
                    .and_then(|span| span.extensions().get::<SpanTimingId>().map(|id| id.0))
            })
            .unwrap_or(0);

        let name = format!("{}::{}", attrs.metadata().target(), attrs.metadata().name());
        self.capture
            .record_event(name, internal_id, parent_id, SpanEventType::Enter);
    }

    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id)
            && let Some(timing_id) = span.extensions().get::<SpanTimingId>()
        {
            let name = format!("{}::{}", span.metadata().target(), span.metadata().name());
            self.capture
                .record_event(name, timing_id.0, 0, SpanEventType::Enter);
        }
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id)
            && let Some(timing_id) = span.extensions().get::<SpanTimingId>()
        {
            let name = format!("{}::{}", span.metadata().target(), span.metadata().name());
            self.capture
                .record_event(name, timing_id.0, 0, SpanEventType::Exit);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id)
            && let Some(timing_id) = span.extensions().get::<SpanTimingId>()
        {
            let name = format!("{}::{}", span.metadata().target(), span.metadata().name());
            self.capture
                .record_event(name, timing_id.0, 0, SpanEventType::Close);
        }
    }
}

// ============================================================================
// Span Correlation Report
// ============================================================================

/// A span that was active during CPU profiling.
#[derive(Debug, Clone)]
pub struct ActiveSpan {
    /// Span name
    pub name: String,
    /// Total time the span was active (microseconds)
    pub total_time_us: u64,
    /// Number of times the span was entered
    pub enter_count: u64,
    /// Percentage of profile duration this span was active
    pub percentage: f64,
}

/// Report correlating CPU profile with span timing.
#[derive(Debug)]
pub struct SpanCorrelationReport {
    /// CPU profile result
    pub profile: ProfileResult,
    /// Total profile duration in microseconds
    pub duration_us: u64,
    /// Spans that were active during profiling, sorted by total time
    pub active_spans: Vec<ActiveSpan>,
    /// Raw span events (for advanced analysis)
    pub events: Vec<SpanEvent>,
}

impl std::fmt::Display for SpanCorrelationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== CPU Profile with Span Correlation ===")?;
        writeln!(f, "Duration: {:.2}ms", self.duration_us as f64 / 1000.0)?;
        writeln!(f, "Samples: {}", self.profile.sample_count)?;
        writeln!(f, "Flamegraph: {}", self.profile.flamegraph_path.display())?;
        writeln!(f)?;
        writeln!(f, "Active Spans (by time):")?;
        writeln!(f, "{:-<60}", "")?;

        for span in &self.active_spans {
            writeln!(
                f,
                "{:50} {:>6.1}% ({:.2}ms, {} entries)",
                truncate_span_name(&span.name, 50),
                span.percentage,
                span.total_time_us as f64 / 1000.0,
                span.enter_count
            )?;
        }

        if self.active_spans.is_empty() {
            writeln!(f, "(no spans recorded during profiling)")?;
        }

        Ok(())
    }
}

fn truncate_span_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("...{}", &name[name.len() - max_len + 3..])
    }
}

/// Analyzes span events to compute active span statistics.
fn analyze_span_events(events: &[SpanEvent], duration_us: u64) -> Vec<ActiveSpan> {
    use std::collections::HashMap;

    // Track active time per span name
    #[derive(Default)]
    struct SpanStats {
        total_time_us: u64,
        enter_count: u64,
        last_enter_time: Option<u64>,
    }

    let mut stats: HashMap<String, SpanStats> = HashMap::new();

    for event in events {
        let entry = stats.entry(event.name.clone()).or_default();

        match event.event_type {
            SpanEventType::Enter => {
                entry.enter_count += 1;
                entry.last_enter_time = Some(event.timestamp_us);
            }
            SpanEventType::Exit | SpanEventType::Close => {
                if let Some(enter_time) = entry.last_enter_time.take() {
                    entry.total_time_us += event.timestamp_us.saturating_sub(enter_time);
                }
            }
        }
    }

    // Handle spans that were still active at the end
    for entry in stats.values_mut() {
        if let Some(enter_time) = entry.last_enter_time.take() {
            entry.total_time_us += duration_us.saturating_sub(enter_time);
        }
    }

    let mut active_spans: Vec<_> = stats
        .into_iter()
        .filter(|(_, s)| s.total_time_us > 0 || s.enter_count > 0)
        .map(|(name, s)| ActiveSpan {
            name,
            total_time_us: s.total_time_us,
            enter_count: s.enter_count,
            percentage: if duration_us > 0 {
                (s.total_time_us as f64 / duration_us as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    // Sort by total time descending
    active_spans.sort_by(|a, b| b.total_time_us.cmp(&a.total_time_us));

    active_spans
}

// ============================================================================
// Basic Traced Profiling (existing functionality)
// ============================================================================

/// Holds the active profiling span guard.
struct ProfilingSpanGuard {
    span: Mutex<Option<Span>>,
}

/// Extension trait for tracing-aware CPU profiling.
///
/// This trait wraps [`ProfilingExt`](tauri_plugin_profiling::ProfilingExt) methods
/// with automatic span creation and logging.
pub trait TracedProfilingExt<R: Runtime> {
    /// Starts CPU profiling with automatic span creation and logging.
    fn start_cpu_profile_traced(&self) -> ProfilingResult<()>;

    /// Starts CPU profiling with options, automatic span creation, and logging.
    fn start_cpu_profile_traced_with_options(&self, options: StartOptions) -> ProfilingResult<()>;

    /// Stops CPU profiling, closes the span, and logs results.
    fn stop_cpu_profile_traced(&self) -> ProfilingResult<ProfileResult>;
}

impl<R: Runtime, T: Manager<R>> TracedProfilingExt<R> for T {
    fn start_cpu_profile_traced(&self) -> ProfilingResult<()> {
        start_traced_impl(self.app_handle(), None)
    }

    fn start_cpu_profile_traced_with_options(&self, options: StartOptions) -> ProfilingResult<()> {
        start_traced_impl(self.app_handle(), Some(options))
    }

    fn stop_cpu_profile_traced(&self) -> ProfilingResult<ProfileResult> {
        let result = self.app_handle().stop_cpu_profile()?;

        tracing::info!(
            samples = result.sample_count,
            duration_ms = result.duration_ms,
            flamegraph = %result.flamegraph_path.display(),
            "CPU profiling stopped"
        );

        if let Some(state) = self.app_handle().try_state::<ProfilingSpanGuard>()
            && let Ok(mut guard) = state.span.lock()
            && let Some(span) = guard.take()
        {
            drop(span);
        }

        Ok(result)
    }
}

fn start_traced_impl<R: Runtime>(
    app: &AppHandle<R>,
    options: Option<StartOptions>,
) -> ProfilingResult<()> {
    let frequency = options.as_ref().and_then(|o| o.frequency).unwrap_or(100);

    let span = tracing::info_span!("cpu_profile", frequency = frequency);
    tracing::info!(frequency = frequency, "CPU profiling started");

    match options {
        Some(opts) => app.start_cpu_profile_with_options(opts)?,
        None => app.start_cpu_profile()?,
    }

    match app.try_state::<ProfilingSpanGuard>() {
        Some(state) => {
            if let Ok(mut guard) = state.span.lock() {
                *guard = Some(span);
            }
        }
        None => {
            app.manage(ProfilingSpanGuard {
                span: Mutex::new(Some(span)),
            });
        }
    }

    Ok(())
}

// ============================================================================
// Span-Aware Profiling (with correlation)
// ============================================================================

/// Extension trait for CPU profiling with span correlation.
///
/// Requires a [`SpanTimingCapture`] to be registered in Tauri state.
pub trait SpanAwareProfilingExt<R: Runtime> {
    /// Starts CPU profiling and span timing capture.
    fn start_span_aware_profile(&self) -> ProfilingResult<()>;

    /// Starts CPU profiling with options and span timing capture.
    fn start_span_aware_profile_with_options(&self, options: StartOptions) -> ProfilingResult<()>;

    /// Stops profiling and returns a correlation report.
    fn stop_span_aware_profile(&self) -> ProfilingResult<SpanCorrelationReport>;
}

impl<R: Runtime, T: Manager<R>> SpanAwareProfilingExt<R> for T {
    fn start_span_aware_profile(&self) -> ProfilingResult<()> {
        start_span_aware_impl(self.app_handle(), None)
    }

    fn start_span_aware_profile_with_options(&self, options: StartOptions) -> ProfilingResult<()> {
        start_span_aware_impl(self.app_handle(), Some(options))
    }

    fn stop_span_aware_profile(&self) -> ProfilingResult<SpanCorrelationReport> {
        // Stop CPU profiling
        let profile = self.app_handle().stop_cpu_profile()?;

        // Stop span capture and get events
        let events = if let Some(capture) = self.app_handle().try_state::<Arc<SpanTimingCapture>>()
        {
            capture.stop_capture()
        } else {
            Vec::new()
        };

        let duration_us = profile.duration_ms * 1000;
        let active_spans = analyze_span_events(&events, duration_us);

        tracing::info!(
            samples = profile.sample_count,
            duration_ms = profile.duration_ms,
            spans_recorded = events.len(),
            active_spans = active_spans.len(),
            flamegraph = %profile.flamegraph_path.display(),
            "Span-aware CPU profiling stopped"
        );

        Ok(SpanCorrelationReport {
            profile,
            duration_us,
            active_spans,
            events,
        })
    }
}

fn start_span_aware_impl<R: Runtime>(
    app: &AppHandle<R>,
    options: Option<StartOptions>,
) -> ProfilingResult<()> {
    // Start span capture if available
    if let Some(capture) = app.try_state::<Arc<SpanTimingCapture>>() {
        capture.start_capture();
    } else {
        tracing::warn!(
            "SpanTimingCapture not found in state - span correlation will be unavailable"
        );
    }

    let frequency = options.as_ref().and_then(|o| o.frequency).unwrap_or(100);
    tracing::info!(frequency = frequency, "Span-aware CPU profiling started");

    match options {
        Some(opts) => app.start_cpu_profile_with_options(opts)?,
        None => app.start_cpu_profile()?,
    }

    Ok(())
}
