---
"tracing": minor
"tracing-js": minor
---

Add `with_max_file_size()` builder method for size-based log rotation. Use `MaxFileSize::kb()`, `MaxFileSize::mb()`, or `MaxFileSize::gb()` for convenient size specification. Size-based rotation can be combined with time-based rotation.
