---
"tracing": minor
"tracing-js": minor
---

Add flamegraph profiling support via new `flamegraph` feature. Enable span recording with `with_flamegraph()` builder method, then generate visualizations using `generateFlamegraph()` (collapses identical stacks) or `generateFlamechart()` (preserves event ordering) from JavaScript.
