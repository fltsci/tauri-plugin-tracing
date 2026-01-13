# Tauri Plugin Tracing

A Tauri plugin that integrates the Rust `tracing` crate for structured logging, bridging logs between your Rust backend and JavaScript frontend.

## Installation

### Rust

```toml
[dependencies]
tauri-plugin-tracing = "0.2"
```

### JavaScript

```bash
npm install @fltsci/tauri-plugin-tracing
# or
pnpm add @fltsci/tauri-plugin-tracing
```

## Quick Start

```rust
use tauri_plugin_tracing::{Builder, LevelFilter};

fn main() {
    tauri::Builder::default()
        .plugin(
            Builder::new()
                .with_max_level(LevelFilter::DEBUG)
                .with_default_subscriber()
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```typescript
import { info, debug, error, attachConsole } from '@fltsci/tauri-plugin-tracing';

// Forward Rust logs to browser console
await attachConsole();

// Log from JavaScript
info('Application started');
debug('Debug details', { user: 'alice' });
error('Something went wrong');
```

## Features

### Cargo Features

- **`colored`** - ANSI color output in terminal
- **`specta`** - TypeScript type generation
- **`timing`** - Performance timing with `time()` / `timeEnd()`
- **`flamegraph`** - Performance profiling with flamegraph/flamechart generation

### Log Targets

Configure where logs are written:

```rust
use tauri_plugin_tracing::{Builder, Target};

Builder::new()
    .targets([
        Target::Stdout,
        Target::Webview,
        Target::LogDir { file_name: None },
    ])
    .with_default_subscriber()
    .build()
```

- **Stdout** / **Stderr** - Terminal output
- **Webview** - Forward to JavaScript via events
- **LogDir** - Platform log directory (`~/Library/Logs/` on macOS)
- **Folder** - Custom directory

### File Logging

```rust
Builder::new()
    .with_file_logging()                              // Enable file logging
    .with_rotation(Rotation::Daily)                   // Rotate daily
    .with_rotation_strategy(RotationStrategy::KeepSome(7))  // Keep 7 files
    .with_max_file_size(MaxFileSize::mb(10))          // Rotate at 10 MB
    .with_default_subscriber()
    .build()
```

### Per-Module Filtering

```rust
Builder::new()
    .with_max_level(LevelFilter::INFO)
    .with_target("my_app::database", LevelFilter::DEBUG)
    .with_target("hyper", LevelFilter::WARN)
    .with_default_subscriber()
    .build()
```

### Custom Formatting

```rust
use tauri_plugin_tracing::{Builder, LogFormat};

Builder::new()
    .with_format(LogFormat::Compact)  // or Full, Pretty
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_default_subscriber()
    .build()
```

### Custom Tracing Layers

```rust
use tracing_subscriber::Layer;

let otel_layer = tracing_opentelemetry::layer().boxed();

Builder::new()
    .with_layer(otel_layer)
    .with_default_subscriber()
    .build()
```

### Performance Timing

Requires the `timing` feature.

```typescript
import { time, timeEnd } from '@fltsci/tauri-plugin-tracing';

time('database-query');
const results = await db.query('SELECT * FROM users');
timeEnd('database-query');  // Logs: "database-query: 42.123ms"
```

### Performance Profiling

Requires the `flamegraph` feature.

```rust
Builder::new()
    .with_flamegraph()
    .with_default_subscriber()
    .build()
```

```typescript
import { generateFlamegraph, generateFlamechart } from '@fltsci/tauri-plugin-tracing';

// Generate a flamegraph (collapses identical stack frames)
const flamegraphPath = await generateFlamegraph();

// Generate a flamechart (preserves event ordering)
const flamechartPath = await generateFlamechart();
```

## Custom Subscriber Setup

For advanced use cases, compose your own subscriber:

```rust
use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter};
use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};

let builder = Builder::new()
    .with_max_level(LevelFilter::DEBUG)
    .with_target("hyper", LevelFilter::WARN);

let filter = builder.build_filter();

tauri::Builder::default()
    .plugin(builder.build())
    .setup(move |app| {
        let subscriber = Registry::default()
            .with(fmt::layer())
            .with(WebviewLayer::new(app.handle().clone()))
            .with(filter);
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Custom Subscriber with File Logging

Use `tracing_appender` (re-exported by this crate) for file logging with custom subscribers:

```rust
use tauri_plugin_tracing::{Builder, WebviewLayer, LevelFilter, tracing_appender};
use tracing_subscriber::{Registry, layer::SubscriberExt, fmt};

let builder = Builder::new().with_max_level(LevelFilter::DEBUG);
let filter = builder.build_filter();

tauri::Builder::default()
    .plugin(builder.build())
    .setup(move |app| {
        let log_dir = app.path().app_log_dir()?;
        let file_appender = tracing_appender::rolling::daily(&log_dir, "app");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Store guard in Tauri state to keep file logging active
        app.manage(guard);

        let subscriber = Registry::default()
            .with(fmt::layer())
            .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
            .with(WebviewLayer::new(app.handle().clone()))
            .with(filter);
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Early Initialization

For maximum control, initialize tracing before creating the Tauri app:

```rust
use tauri_plugin_tracing::{Builder, StripAnsiWriter, tracing_appender};
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry};

fn setup_logger() -> Builder {
    let log_dir = std::env::temp_dir().join("my-app");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    std::mem::forget(guard); // Keep file logging active for app lifetime

    let targets = Targets::new()
        .with_default(Level::DEBUG)
        .with_target("hyper", Level::WARN)
        .with_target("reqwest", Level::WARN);

    registry()
        .with(fmt::layer().with_ansi(true))
        .with(fmt::layer().with_writer(StripAnsiWriter::new(non_blocking)).with_ansi(false))
        .with(targets)
        .init();

    Builder::new() // Return minimal builder - logging is already configured
}

fn main() {
    let builder = setup_logger();
    tauri::Builder::default()
        .plugin(builder.build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## JavaScript API

### Logging

```typescript
import { trace, debug, info, warn, error } from '@fltsci/tauri-plugin-tracing';

trace('Very verbose info');
debug('Debug details');
info('General info');
warn('Warning');
error('Error');
```

### Event Listeners

```typescript
import { attachConsole, attachLogger } from '@fltsci/tauri-plugin-tracing';

// Forward all Rust logs to browser console
const unlisten = await attachConsole();

// Custom log handler
const unlisten = await attachLogger(({ level, message }) => {
    // Send to external service, store locally, etc.
});
```

## License

MIT
