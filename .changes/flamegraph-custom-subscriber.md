---
"tracing": minor
"tracing-js": minor
---

Add `create_flame_layer_with_path()` and `FlameExt` trait for early initialization with flamegraph profiling. This enables initializing tracing before Tauri starts while still supporting frontend flamegraph generation via `register_flamegraph()`.
