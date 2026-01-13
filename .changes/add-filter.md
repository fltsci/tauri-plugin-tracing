---
"tracing": minor
"tracing-js": minor
---

Add `filter()` builder method for metadata-based log filtering. Accepts a closure that receives event metadata and returns `true` to include the log. Applied in addition to level and target filters when using `with_default_subscriber()`.
