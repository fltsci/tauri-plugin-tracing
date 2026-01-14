---
"tracing": minor
"tracing-js": minor
---

Release 0.2.0 stable.

**Highlights:**
- Add `flamegraph` feature for performance profiling with flamegraph/flamechart generation
- Add `WebviewLayer` for custom subscriber setups
- Add file logging with rotation (`with_file_logging()`, `with_rotation()`, `with_max_file_size()`)
- Add `StripAnsiWriter` for clean file output when using colored terminal output
- Add custom filter support (`filter()`) and custom layer support (`with_layer()`)
- Add format options (`with_format()`, `with_file()`, `with_line_number()`, etc.)
- Add timezone strategy for log timestamps
- Add per-target log level filtering (`with_target()`)
- Add `takeoverConsole()` for full console integration (JS → Rust → browser)
- Add `interceptConsole()` and `attachConsole()` for flexible console routing
- Add early initialization support for flamegraph (`create_flame_layer_with_path()`)
- Add clippy lints to deny `unwrap()`/`expect()` in production code
- Remove `timing` feature (use native `console.time()` or tracing spans instead)
