---
"tracing": minor
"tracing-js": minor
---

Add flamegraph support for custom subscribers and early initialization:
- `create_flame_layer_with_path()` creates a flame layer without requiring an AppHandle
- `FlameGuard` struct holds the flush guard and output path
- `FlameExt` trait adds `register_flamegraph()` to AppHandle for late registration

This enables initializing tracing before Tauri starts while still supporting frontend flamegraph generation.
