---
"tracing": minor
"tracing-js": minor
---

Add flamegraph profiling support via new `flamegraph` feature. Enable span recording with `with_flamegraph()` builder method, then generate visualizations using `generateFlamegraph()` (collapses identical stacks) or `generateFlamechart()` (preserves event ordering) from JavaScript.

**Breaking:** Remove `timing` feature (`time()`/`timeEnd()` APIs). Use native `console.time()` for JS timing or tracing spans for Rust timing instead.
