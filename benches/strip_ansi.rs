use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::io::Write;
use tauri_plugin_tracing::StripAnsiWriter;
use tracing_subscriber::fmt::MakeWriter;

// Test inputs of varying sizes and ANSI density
fn no_ansi_short() -> &'static [u8] {
    b"Hello, world!"
}

fn no_ansi_medium() -> &'static [u8] {
    b"2024-01-15T10:30:00.000Z INFO my_app::module: Processing request id=12345 user=alice"
}

fn no_ansi_long() -> &'static [u8] {
    b"2024-01-15T10:30:00.000Z DEBUG my_app::database::connection_pool: Acquired connection from pool connection_id=42 pool_size=10 active_connections=5 idle_connections=5 wait_time_ms=0.123 query=\"SELECT * FROM users WHERE id = $1\""
}

fn with_ansi_simple() -> &'static [u8] {
    b"\x1b[32mgreen\x1b[0m"
}

fn with_ansi_log_line() -> &'static [u8] {
    b"\x1b[2m2024-01-15T10:30:00.000Z\x1b[0m \x1b[32m INFO\x1b[0m \x1b[2mmy_app::module\x1b[0m\x1b[2m:\x1b[0m Processing request"
}

fn with_ansi_complex() -> &'static [u8] {
    b"\x1b[2m2024-01-15T10:30:00.000Z\x1b[0m \x1b[1;31m ERROR\x1b[0m \x1b[2mmy_app::handler\x1b[0m\x1b[2m:\x1b[0m \x1b[1;33mConnection failed\x1b[0m error=\"\x1b[31mTimeout after 30s\x1b[0m\" retries=\x1b[36m3\x1b[0m"
}

fn bench_strip_ansi_writer(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_ansi_writer");

    // No ANSI - should use fast path
    for (name, input) in [
        ("no_ansi_short", no_ansi_short()),
        ("no_ansi_medium", no_ansi_medium()),
        ("no_ansi_long", no_ansi_long()),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("fast_path", name), input, |b, input| {
            let writer = StripAnsiWriter::new(Vec::with_capacity(512));
            b.iter(|| {
                let mut guard = writer.make_writer();
                guard.write_all(black_box(input)).unwrap();
            });
        });
    }

    // With ANSI - uses slow path
    for (name, input) in [
        ("with_ansi_simple", with_ansi_simple()),
        ("with_ansi_log_line", with_ansi_log_line()),
        ("with_ansi_complex", with_ansi_complex()),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("slow_path", name), input, |b, input| {
            let writer = StripAnsiWriter::new(Vec::with_capacity(512));
            b.iter(|| {
                let mut guard = writer.make_writer();
                guard.write_all(black_box(input)).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_ansi_throughput");

    // Simulate realistic logging workload - many small writes
    let log_lines: Vec<&[u8]> = vec![
        b"2024-01-15T10:30:00.000Z INFO request started",
        b"\x1b[32m INFO\x1b[0m processing",
        b"2024-01-15T10:30:00.001Z DEBUG step 1 complete",
        b"\x1b[33m WARN\x1b[0m slow query",
        b"2024-01-15T10:30:00.002Z INFO request complete",
    ];

    let total_bytes: usize = log_lines.iter().map(|l| l.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("mixed_log_batch", |b| {
        let writer = StripAnsiWriter::new(Vec::with_capacity(1024));
        b.iter(|| {
            let mut guard = writer.make_writer();
            for line in &log_lines {
                guard.write_all(black_box(line)).unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_strip_ansi_writer, bench_throughput);
criterion_main!(benches);
