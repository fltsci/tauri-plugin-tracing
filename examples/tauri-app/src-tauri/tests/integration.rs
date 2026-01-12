//! Integration tests for tauri-plugin-tracing.
//!
//! These tests exercise functionality that requires a Tauri context,
//! which cannot be tested via doctests in the main crate.

use tauri_plugin_tracing::{Builder, CallStack, CallStackLine, LevelFilter, LogLevel};

#[test]
fn builder_default() {
    // Default builder should create successfully
    let _plugin = Builder::new().build::<tauri::Wry>();
}

#[test]
fn builder_with_max_level() {
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_target() {
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::INFO)
        .with_target("my_app::database", LevelFilter::DEBUG)
        .with_target("hyper", LevelFilter::WARN)
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_colors() {
    let _plugin = Builder::new()
        .with_colors()
        .with_max_level(LevelFilter::TRACE)
        .build::<tauri::Wry>();
}

#[test]
fn builder_full_configuration() {
    // Test a fully configured builder like the example app uses
    let _plugin = Builder::new()
        .with_colors()
        .with_max_level(LevelFilter::TRACE)
        .with_target("tao::platform_impl", LevelFilter::WARN)
        .with_target("wry", LevelFilter::WARN)
        .build::<tauri::Wry>();
}

#[test]
fn log_level_default() {
    assert!(matches!(LogLevel::default(), LogLevel::Info));
}

#[test]
fn log_level_to_tracing() {
    let level: tracing::Level = LogLevel::Trace.into();
    assert_eq!(tracing::Level::TRACE, level);

    let level: tracing::Level = LogLevel::Debug.into();
    assert_eq!(tracing::Level::DEBUG, level);

    let level: tracing::Level = LogLevel::Info.into();
    assert_eq!(tracing::Level::INFO, level);

    let level: tracing::Level = LogLevel::Warn.into();
    assert_eq!(tracing::Level::WARN, level);

    let level: tracing::Level = LogLevel::Error.into();
    assert_eq!(tracing::Level::ERROR, level);
}

#[test]
fn log_level_from_tracing() {
    assert!(matches!(
        LogLevel::from(tracing::Level::TRACE),
        LogLevel::Trace
    ));
    assert!(matches!(
        LogLevel::from(tracing::Level::DEBUG),
        LogLevel::Debug
    ));
    assert!(matches!(
        LogLevel::from(tracing::Level::INFO),
        LogLevel::Info
    ));
    assert!(matches!(
        LogLevel::from(tracing::Level::WARN),
        LogLevel::Warn
    ));
    assert!(matches!(
        LogLevel::from(tracing::Level::ERROR),
        LogLevel::Error
    ));
}

#[test]
fn call_stack_line_from_str() {
    let line = CallStackLine::from("at foo (src/app.ts:10:5)");
    assert!(line.contains("foo"));
    assert!(line.contains("src/app.ts"));
}

#[test]
fn call_stack_line_default() {
    let line = CallStackLine::default();
    assert_eq!(line.as_str(), "unknown");
}

#[test]
fn call_stack_line_from_none() {
    let line = CallStackLine::from(None);
    assert_eq!(line.as_str(), "unknown");
}

#[test]
fn call_stack_line_replace() {
    let line = CallStackLine::from("at foo (src/old.ts:10:5)");
    let replaced = line.replace("old", "new");
    assert!(replaced.contains("new.ts"));
    assert!(!replaced.contains("old.ts"));
}

#[test]
fn call_stack_parsing() {
    let stack = CallStack::new(Some(
        "Error\n    at foo (src/app.ts:10:5)\n    at bar (src/lib.ts:20:3)",
    ));

    // file_name returns last component after '/'
    assert_eq!(stack.file_name().as_str(), "lib.ts:20:3)");

    // path returns the last frame
    assert_eq!(stack.path().as_str(), "    at bar (src/lib.ts:20:3)");
}

#[test]
fn call_stack_filters_node_modules() {
    let stack = CallStack::new(Some(
        "Error\n    at node_modules/lib/index.js:1:1\n    at src/app.ts:10:5",
    ));
    let location = stack.location();
    assert!(!location.contains("node_modules"));
    assert!(location.contains("src/app.ts"));
}

#[test]
fn call_stack_filters_native_code() {
    let stack = CallStack::new(Some(
        "Error\n    at forEach@[native code]\n    at src/app.ts:10:5",
    ));
    let location = stack.location();
    assert!(!location.contains("native code"));
    assert!(location.contains("src/app.ts"));
}

#[test]
fn call_stack_empty() {
    let stack = CallStack::new(None);
    // Empty stack results in empty location, which splits to empty file_name
    assert_eq!(stack.file_name().as_str(), "");
    assert_eq!(stack.path().as_str(), "");
    assert_eq!(stack.location().as_str(), "");
}

#[test]
fn call_stack_strips_localhost() {
    let stack = CallStack::new(Some("at http://localhost:1420/src/app.ts:10:5"));
    let location = stack.location();
    assert!(!location.contains("localhost"));
    assert!(location.contains("src/app.ts"));
}
