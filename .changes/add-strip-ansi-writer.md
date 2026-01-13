---
"tracing": minor
"tracing-js": minor
---

Add `StripAnsiWriter` for stripping ANSI escape codes from file writers, and printf-style format expansion (%s, %d, %i, %f, %o, %O) in JS. `StripAnsiWriter` uses a zero-copy fast path with SIMD-accelerated ESC byte detection via memchr.
