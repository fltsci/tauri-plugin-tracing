# Tauri Plugin Tracing

A modified version of `@tauri-apps/plugin-log` that implements the `tracing` crate.

## Roadmap: Feature Parity with tauri-plugin-log

### Core Features

- [x] Log levels (trace, debug, info, warn, error)
- [x] Per-module level filtering (`with_target()`)
- [x] Colored terminal output (`colored` feature)
- [x] iOS native logging (swift_rs)
- [x] Strip ANSI codes from messages
- [x] File logging (LogDir target)
- [x] Custom folder logging
- [x] Log rotation (KeepAll, KeepOne, KeepSome)
- [ ] Max file size configuration
- [ ] Timezone configuration (UTC/local)

### JavaScript API

- [x] `trace()`, `debug()`, `info()`, `warn()`, `error()`
- [x] `attachLogger()` - attach custom log listener
- [x] `attachConsole()` - route logs to browser console
- [x] Webview target (emit events from Rust to JS via WebviewLayer)

### Builder Configuration

- [x] `new()` / `default()`
- [x] `with_max_level()` - global log level
- [x] `with_target()` - per-module levels
- [x] `with_colors()` - colored output
- [x] `with_rotation()` - time-based rotation period
- [x] `with_rotation_strategy()` - file retention policy
- [ ] `targets()` / `target()` - configure log destinations
- [ ] `max_file_size()` - rotation threshold
- [ ] `timezone_strategy()` - timestamp timezone
- [ ] `format()` / `clear_format()` - custom formatting
- [ ] `filter()` - metadata-based filtering
- [ ] Custom tracing layers/targets

### Extra Features (not in tauri-plugin-log)

- [x] `time()` / `timeEnd()` - performance timing (`timing` feature)
- [x] Call stack parsing - JS file location in logs
- [x] Window name in logs - webview label included
- [x] `specta` integration - TypeScript type generation (`specta` feature)

### Known Issues / Future Work

- [x] ~~Find alternative to `set_global_default()`~~ - plugin no longer sets global by default; use `WebviewLayer` + `build_filter()` for custom setup, or `with_default_subscriber()` for convenience
