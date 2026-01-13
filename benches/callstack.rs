use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tauri_plugin_tracing::CallStack;

// Realistic call stack from a Tauri app
const SIMPLE_STACK: &str = r#"Error
    at info (http://localhost:1420/src/utils/logger.ts:42:5)
    at handleClick (http://localhost:1420/src/components/Button.tsx:15:3)
    at onClick (http://localhost:1420/src/App.tsx:28:9)"#;

// Call stack with node_modules (should be filtered)
const STACK_WITH_NODE_MODULES: &str = r#"Error
    at Object.invoke (http://localhost:1420/node_modules/@tauri-apps/api/dist/core.js:123:45)
    at async fetchData (http://localhost:1420/node_modules/some-lib/index.js:10:5)
    at handleSubmit (http://localhost:1420/src/components/Form.tsx:55:12)
    at processForm (http://localhost:1420/src/utils/forms.ts:20:3)"#;

// Deep call stack
const DEEP_STACK: &str = r#"Error
    at level10 (http://localhost:1420/src/deep/a.ts:10:1)
    at level9 (http://localhost:1420/src/deep/b.ts:20:1)
    at level8 (http://localhost:1420/src/deep/c.ts:30:1)
    at level7 (http://localhost:1420/src/deep/d.ts:40:1)
    at level6 (http://localhost:1420/src/deep/e.ts:50:1)
    at level5 (http://localhost:1420/src/deep/f.ts:60:1)
    at level4 (http://localhost:1420/src/deep/g.ts:70:1)
    at level3 (http://localhost:1420/src/deep/h.ts:80:1)
    at level2 (http://localhost:1420/src/deep/i.ts:90:1)
    at level1 (http://localhost:1420/src/deep/j.ts:100:1)"#;

// Empty/minimal stack
const EMPTY_STACK: &str = "";
const MINIMAL_STACK: &str = "Error\n    at main (src/main.ts:1:1)";

fn bench_callstack_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("callstack_parsing");

    group.bench_function("simple", |b| {
        b.iter(|| CallStack::from(black_box(Some(SIMPLE_STACK))));
    });

    group.bench_function("with_node_modules", |b| {
        b.iter(|| CallStack::from(black_box(Some(STACK_WITH_NODE_MODULES))));
    });

    group.bench_function("deep", |b| {
        b.iter(|| CallStack::from(black_box(Some(DEEP_STACK))));
    });

    group.bench_function("empty", |b| {
        b.iter(|| CallStack::from(black_box(Some(EMPTY_STACK))));
    });

    group.bench_function("minimal", |b| {
        b.iter(|| CallStack::from(black_box(Some(MINIMAL_STACK))));
    });

    group.bench_function("none", |b| {
        b.iter(|| CallStack::from(black_box(None::<&str>)));
    });

    group.finish();
}

fn bench_callstack_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("callstack_methods");

    // Pre-parse the stacks
    let simple = CallStack::from(Some(SIMPLE_STACK));
    let with_node_modules = CallStack::from(Some(STACK_WITH_NODE_MODULES));
    let deep = CallStack::from(Some(DEEP_STACK));

    // Benchmark location() - most expensive, joins all frames
    group.bench_function("location/simple", |b| {
        b.iter(|| black_box(&simple).location());
    });

    group.bench_function("location/with_node_modules", |b| {
        b.iter(|| black_box(&with_node_modules).location());
    });

    group.bench_function("location/deep", |b| {
        b.iter(|| black_box(&deep).location());
    });

    // Benchmark path() - calls location() then splits
    group.bench_function("path/simple", |b| {
        b.iter(|| black_box(&simple).path());
    });

    group.bench_function("path/with_node_modules", |b| {
        b.iter(|| black_box(&with_node_modules).path());
    });

    // Benchmark file_name() - calls location() then splits by /
    group.bench_function("file_name/simple", |b| {
        b.iter(|| black_box(&simple).file_name());
    });

    group.bench_function("file_name/with_node_modules", |b| {
        b.iter(|| black_box(&with_node_modules).file_name());
    });

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("callstack_full_pipeline");

    // Simulate what happens in a log command: parse + extract location
    group.bench_function("parse_and_location", |b| {
        b.iter(|| {
            let stack = CallStack::from(black_box(Some(SIMPLE_STACK)));
            stack.location()
        });
    });

    group.bench_function("parse_and_file_name", |b| {
        b.iter(|| {
            let stack = CallStack::from(black_box(Some(SIMPLE_STACK)));
            stack.file_name()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_callstack_parsing,
    bench_callstack_methods,
    bench_full_pipeline
);
criterion_main!(benches);
