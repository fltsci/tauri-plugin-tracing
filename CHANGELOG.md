# Changelog

## \[0.2.0-canary.20]

- [`061b015`](https://github.com/fltsci/tauri-plugin-tracing/commit/061b0159e4b69055d6adfa98a3c40712640fc58b) ([#84](https://github.com/fltsci/tauri-plugin-tracing/pull/84) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Clarify that the flamegraph feature measures span timing (wall-clock time), not CPU time. Updated documentation with appropriate use cases and limitations.
- [`675d209`](https://github.com/fltsci/tauri-plugin-tracing/commit/675d2095ae6edb7d4333cd5225d29c3914d39469) ([#82](https://github.com/fltsci/tauri-plugin-tracing/pull/82) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Switch to public package publishing: Rust on crates.io, npm with public access.

## \[0.2.0-canary.19]

- [`7099260`](https://github.com/fltsci/tauri-plugin-tracing/commit/70992603fbf50a40968fe0b475725f0b0453793f) ([#80](https://github.com/fltsci/tauri-plugin-tracing/pull/80) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `takeoverConsole()` function that completely takes over the webview console, routing all logs through Rust tracing and back to the original console methods.

## \[0.2.0-canary.18]

- [`680f9c9`](https://github.com/fltsci/tauri-plugin-tracing/commit/680f9c9b39207db5964f297a792402a45044f637) ([#78](https://github.com/fltsci/tauri-plugin-tracing/pull/78) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add clippy lints to deny `unwrap()` and `expect()` in production code, preventing accidental panics.

## \[0.2.0-canary.17]

- [`60897d0`](https://github.com/fltsci/tauri-plugin-tracing/commit/60897d067e62e23677aecdc565090b6681dba9f4) ([#76](https://github.com/fltsci/tauri-plugin-tracing/pull/76) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add flamegraph support for custom subscribers and early initialization:

  - `create_flame_layer_with_path()` creates a flame layer without requiring an AppHandle
  - `FlameGuard` struct holds the flush guard and output path
  - `FlameExt` trait adds `register_flamegraph()` to AppHandle for late registration

  This enables initializing tracing before Tauri starts while still supporting frontend flamegraph generation.

## \[0.2.0-canary.16]

- [`1bea996`](https://github.com/fltsci/tauri-plugin-tracing/commit/1bea996fe8a9f6e677151c124bfb10e4db4d1615) ([#74](https://github.com/fltsci/tauri-plugin-tracing/pull/74) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add ACL permissions for flamegraph commands.

## \[0.2.0-canary.15]

- [`fe47ac1`](https://github.com/fltsci/tauri-plugin-tracing/commit/fe47ac1a49f9292c78b3b96f909bb9e4cec1a428) ([#71](https://github.com/fltsci/tauri-plugin-tracing/pull/71) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix broken rustdoc intra-doc links.

## \[0.2.0-canary.14]

- [`075cdbd`](https://github.com/fltsci/tauri-plugin-tracing/commit/075cdbd300eb064d361faff9d82c54e9274af0ad) ([#57](https://github.com/fltsci/tauri-plugin-tracing/pull/57) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add flamegraph profiling support via new `flamegraph` feature. Enable span recording with `with_flamegraph()` builder method, then generate visualizations using `generateFlamegraph()` (collapses identical stacks) or `generateFlamechart()` (preserves event ordering) from JavaScript.

  **Breaking:** Remove `timing` feature (`time()`/`timeEnd()` APIs). Use native `console.time()` for JS timing or tracing spans for Rust timing instead.

## \[0.2.0-canary.13]

- [`4fa8668`](https://github.com/fltsci/tauri-plugin-tracing/commit/4fa86684476164d57c217f0ae9bd19aaa6207d49) ([#68](https://github.com/fltsci/tauri-plugin-tracing/pull/68) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix documentation incorrectly stating file logging requires default subscriber. Add examples for custom subscriber file logging and early initialization patterns.

## \[0.2.0-canary.12]

- [`db4978d`](https://github.com/fltsci/tauri-plugin-tracing/commit/db4978ddb3c0ba105d27cacd842dc33d2b3beafa) ([#64](https://github.com/fltsci/tauri-plugin-tracing/pull/64) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `StripAnsiWriter` for stripping ANSI escape codes from file writers, and printf-style format expansion (%s, %d, %i, %f, %o, %O) in JS. `StripAnsiWriter` uses a zero-copy fast path with SIMD-accelerated ESC byte detection via memchr.

## \[0.2.0-canary.11]

- [`9eba679`](https://github.com/fltsci/tauri-plugin-tracing/commit/9eba679389dec8bedc60b83711210568496ed935) ([#62](https://github.com/fltsci/tauri-plugin-tracing/pull/62) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add automatic docs upload to Kellnr after publish.

## \[0.2.0-canary.10]

- [`4155d67`](https://github.com/fltsci/tauri-plugin-tracing/commit/4155d6762dac9d7084c5e4ef5e445b26a46f4e22) ([#60](https://github.com/fltsci/tauri-plugin-tracing/pull/60) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix doctests to be compile-checked instead of ignored, eliminating "this example is not tested" warnings.

## \[0.2.0-canary.9]

- [`f364659`](https://github.com/fltsci/tauri-plugin-tracing/commit/f364659631d66e9e47b2fb3e1364222660483978) ([#58](https://github.com/fltsci/tauri-plugin-tracing/pull/58) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Rewrote README with comprehensive documentation including installation instructions, quick start guide, feature documentation with code examples, and JavaScript API reference.

## \[0.2.0-canary.8]

- [`c52d634`](https://github.com/fltsci/tauri-plugin-tracing/commit/c52d6343503727879921e78afc0a124b04eee6a3) ([#56](https://github.com/fltsci/tauri-plugin-tracing/pull/56) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `with_layer()` method and `BoxedLayer` type alias for custom tracing layer support. Users can now add their own `tracing_subscriber::Layer` implementations to the subscriber stack.
- [`d0876a5`](https://github.com/fltsci/tauri-plugin-tracing/commit/d0876a5648008197ef7245f99ca1374bf61de5d5) ([#46](https://github.com/fltsci/tauri-plugin-tracing/pull/46) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add file logging support with daily rotation via tracing-appender. New builder methods: `with_file_logging()` for platform-standard log directories, `with_log_dir()` for custom paths.
- [`e893585`](https://github.com/fltsci/tauri-plugin-tracing/commit/e8935857798f97110d8c4ed7212db1fa9da64ac8) ([#54](https://github.com/fltsci/tauri-plugin-tracing/pull/54) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `filter()` builder method for metadata-based log filtering. Accepts a closure that receives event metadata and returns `true` to include the log. Applied in addition to level and target filters when using `with_default_subscriber()`.
- [`b3eb7d3`](https://github.com/fltsci/tauri-plugin-tracing/commit/b3eb7d37be90ec3874fa217f47e2ccf31df97533) ([#55](https://github.com/fltsci/tauri-plugin-tracing/pull/55) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add log formatting options via `with_format()` and related methods. Configure output style (`LogFormat::Full`, `Compact`, or `Pretty`) and control what information is displayed: `with_file()`, `with_line_number()`, `with_thread_ids()`, `with_thread_names()`, `with_target_display()`, `with_level()`. Also adds `FormatOptions` struct and `configured_format()`/`configured_format_options()` getters.
- [`5254fb5`](https://github.com/fltsci/tauri-plugin-tracing/commit/5254fb5b1363f060a77861d16aa5e12ee212232b) ([#48](https://github.com/fltsci/tauri-plugin-tracing/pull/48) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add log rotation support with configurable time-based rotation periods (Daily, Hourly, Minutely, Never) and retention strategies (KeepAll, KeepOne, KeepSome). New builder methods: `with_rotation()` for rotation period, `with_rotation_strategy()` for file retention policy.
- [`7800566`](https://github.com/fltsci/tauri-plugin-tracing/commit/78005664090ebebf3f5e24d471a1ca0842377b36) ([#52](https://github.com/fltsci/tauri-plugin-tracing/pull/52) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `with_max_file_size()` builder method for size-based log rotation. Use `MaxFileSize::kb()`, `MaxFileSize::mb()`, or `MaxFileSize::gb()` for convenient size specification. Size-based rotation can be combined with time-based rotation.
- [`9c4c895`](https://github.com/fltsci/tauri-plugin-tracing/commit/9c4c895671307fc06665fbedd5dd16de5df6452f) ([#51](https://github.com/fltsci/tauri-plugin-tracing/pull/51) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `Target` enum and `target()`/`targets()`/`clear_targets()` builder methods to configure log destinations (Stdout, Stderr, Webview, LogDir, Folder). Add `configured_targets()`, `configured_rotation()`, and `configured_rotation_strategy()` getters for querying builder configuration.
- [`f0e4b49`](https://github.com/fltsci/tauri-plugin-tracing/commit/f0e4b4950aff6ee1cfaca7c4d66d9d1fcefa91f4) ([#53](https://github.com/fltsci/tauri-plugin-tracing/pull/53) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `with_timezone_strategy()` builder method to configure timestamp timezone. Use `TimezoneStrategy::Utc` (default) for UTC timestamps or `TimezoneStrategy::Local` for local time with the system's timezone offset.
- [`225d499`](https://github.com/fltsci/tauri-plugin-tracing/commit/225d49971041b39f1d3b321415653fbca5ee1234) ([#49](https://github.com/fltsci/tauri-plugin-tracing/pull/49) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add WebviewLayer to forward Rust logs to frontend via `tracing://log` events, enabling `attachLogger()` and `attachConsole()`.

## \[0.2.0-canary.7]

- [`dcc07fa`](https://github.com/fltsci/tauri-plugin-tracing/commit/dcc07fa0a5f12d20f513ceff49220566b7f7b050) ([#44](https://github.com/fltsci/tauri-plugin-tracing/pull/44) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add documentation: rustdocs for Rust public API, JSDoc for TypeScript API, integration tests, and feature parity roadmap in README.

## \[0.2.0-canary.6]

- [`8a183d9`](https://github.com/fltsci/tauri-plugin-tracing/commit/8a183d94733787ec0876fdb77f820db2b1d3f1f9) ([#42](https://github.com/fltsci/tauri-plugin-tracing/pull/42) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Include window name in tracing logs.

## \[0.2.0-canary.5]

- [`5549ce1`](https://github.com/fltsci/tauri-plugin-tracing/commit/5549ce1e99a8f80532c20afb3d619ae59096fb50) ([#40](https://github.com/fltsci/tauri-plugin-tracing/pull/40) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Use OIDC publishing for NPM

## \[0.2.0-canary.4]

- [`a8d659b`](https://github.com/fltsci/tauri-plugin-tracing/commit/a8d659bf6a132994cc00a1cd17fb8ec1f5af0d60) ([#38](https://github.com/fltsci/tauri-plugin-tracing/pull/38) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Skipping a version previously published to satisfy NPM publishing rules.

## \[0.2.0-canary.3]

- [`96c37b6`](https://github.com/fltsci/tauri-plugin-tracing/commit/96c37b62b92486c72749607e005e6ed7bb37ac73) ([#36](https://github.com/fltsci/tauri-plugin-tracing/pull/36) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix a lockup possibly caused by the timings' feature's use of a sync mutex.

## \[0.2.0-canary.2]

- [`16ee339`](https://github.com/fltsci/tauri-plugin-tracing/commit/16ee339387ede491544544dd7feaf56d019e6b8c) ([#34](https://github.com/fltsci/tauri-plugin-tracing/pull/34) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add [release channel check](https://docs.npmjs.com/adding-dist-tags-to-packages) to publish job CI.
- [`16ee339`](https://github.com/fltsci/tauri-plugin-tracing/commit/16ee339387ede491544544dd7feaf56d019e6b8c) ([#34](https://github.com/fltsci/tauri-plugin-tracing/pull/34) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) -   Update Tauri to version 2.9.3
  - Update npm dependencies to latest
  - Update example to latest implementation

## \[0.2.0-canary.1]

- [`87086a7`](https://github.com/fltsci/tauri-plugin-tracing/commit/87086a7e7995737d6399a34c6c75ab5938361680) ([#24](https://github.com/fltsci/tauri-plugin-tracing/pull/24) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Update examples and Cargo dependencies.

## \[0.2.0-canary.0]

- [`f865b4a`](https://github.com/fltsci/tauri-plugin-tracing/commit/f865b4aeb0fe23a4b81490059edf7c9f18670ddc) ([#22](https://github.com/fltsci/tauri-plugin-tracing/pull/22) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add timing.

## \[0.1.2-canary.7]

- [`b0fd006`](https://github.com/fltsci/tauri-plugin-tracing/commit/b0fd006759d281a83b6cfb6d54d9e83d76e5bff6) ([#20](https://github.com/fltsci/tauri-plugin-tracing/pull/20) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Refine webview path, change initialization from global to default subscriber

## \[0.1.2-canary.6]

- [`c72e376`](https://github.com/fltsci/tauri-plugin-tracing/commit/c72e37632f064b3f2cc8dea354a690622bf14a4e) ([#18](https://github.com/fltsci/tauri-plugin-tracing/pull/18) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Use sync methods to better emulate the JavaScript console.

## \[0.1.2-canary.5]

- [`ccddfd9`](https://github.com/fltsci/tauri-plugin-tracing/commit/ccddfd9d98c6bf32cb3a6ac77a119efecac1ce92) ([#16](https://github.com/fltsci/tauri-plugin-tracing/pull/16) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `specta` feature for integration with other Tauri & typescript plugins.
- [`ccddfd9`](https://github.com/fltsci/tauri-plugin-tracing/commit/ccddfd9d98c6bf32cb3a6ac77a119efecac1ce92) ([#16](https://github.com/fltsci/tauri-plugin-tracing/pull/16) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix various bugs that have popped up in testing.

## \[0.1.2-canary.4]

- [`549f972`](https://github.com/fltsci/tauri-plugin-tracing/commit/549f972627fc348d8227bcf4c5e1b97e24c639a7) Move NPM package to GitHub packages.

## \[0.1.2-canary.3]

- [`e041ec2`](https://github.com/fltsci/tauri-plugin-tracing/commit/e041ec22c232e78df7e3011ac170376588979cd2) ([#14](https://github.com/fltsci/tauri-plugin-tracing/pull/14) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) In the event webview returns an empty callstack, tauri-plugin-tracing will now use an empty string as the callstack.

## \[0.1.2-canary.2]

- [`6977ca8`](https://github.com/fltsci/tauri-plugin-tracing/commit/6977ca88896d01671048c2b384985b8877c32598) ([#12](https://github.com/fltsci/tauri-plugin-tracing/pull/12) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Users can now filter targets using [tracing_subscriber::Target::with_taget(...)](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/targets/struct.Targets.html) syntax.

## \[0.1.2-canary.1]

- [`6c43912`](https://github.com/fltsci/tauri-plugin-tracing/commit/6c439128ba328244843967d24a1a7531e390c383) ([#9](https://github.com/fltsci/tauri-plugin-tracing/pull/9) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fine tune publishing jobs, use a release branch.

## \[0.1.2-canary.0]

- [`847a016`](https://github.com/fltsci/tauri-plugin-tracing/commit/847a016916292305babbd91bcb6bb5a1a364d764) ([#3](https://github.com/fltsci/tauri-plugin-tracing/pull/3) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Set up publishing to kellnr registry and npm via covector. Add CI jobs for linting and formatting. Adhere to tauri-plugin repo standards wherever practical.
