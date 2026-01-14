---
"tracing": minor
"tracing-js": minor
---

Add CPU profiling integration with span correlation via new `profiling` feature. Includes `TracedProfilingExt` for automatic span/logging around profiles, `SpanTimingLayer` for capturing span timing, and `SpanAwareProfilingExt` for correlating CPU samples with active tracing spans.
