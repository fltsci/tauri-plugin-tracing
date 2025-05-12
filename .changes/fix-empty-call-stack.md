---
"tracing": patch
"tracing-js": patch
---

In the event webview returns an empty callstack, tauri-plugin-tracing will now use an empty string as the callstack.
