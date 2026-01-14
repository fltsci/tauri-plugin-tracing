# Tauri Plugin Tracing

Integrate Rust's `tracing` crate with your Tauri app. Bridge logs between Rust and JavaScript with support for file rotation, custom layers, and span visualization.

## Installation

```toml
[dependencies]
tauri-plugin-tracing = "0.2"
```

```bash
npm install @fltsci/tauri-plugin-tracing
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
import { info, attachConsole } from '@fltsci/tauri-plugin-tracing';

await attachConsole();  // See Rust logs in browser console
info('Hello from JS');  // Send JS logs to Rust
```

## Features

- **Log levels**: trace, debug, info, warn, error
- **Targets**: stdout, stderr, webview, file (with rotation)
- **Filtering**: per-module log levels
- **Custom layers**: OpenTelemetry, Sentry, or any tracing-subscriber layer
- **Span visualization**: flamegraph/flamechart SVG generation (`flamegraph` feature)

### Cargo Features

- `colored` - ANSI color output
- `specta` - TypeScript type generation
- `flamegraph` - Span timing visualization

## Console Integration

```typescript
import { attachConsole, interceptConsole, takeoverConsole } from '@fltsci/tauri-plugin-tracing';

attachConsole();       // Rust logs → browser console
interceptConsole();    // JS console → Rust tracing
takeoverConsole();     // Both directions (full integration)
```

## Documentation

See [docs.rs](https://docs.rs/tauri-plugin-tracing) for the full API reference and advanced usage (custom subscribers, file logging, early initialization).

## License

MIT
