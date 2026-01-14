---
"tracing": minor
"tracing-js": minor
---

Add `takeoverConsole()` function that completely takes over the webview console, routing all logs through Rust tracing and back to the original console methods.
