//! Integration tests for tauri-plugin-tracing.
//!
//! These tests exercise functionality that requires a Tauri context,
//! which cannot be tested via doctests in the main crate.

use std::path::PathBuf;
use tauri_plugin_tracing::{
    Builder, CallStack, CallStackLine, LevelFilter, LogLevel, MaxFileSize, Rotation,
    RotationStrategy, Target,
};

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
    // Test a fully configured builder
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

#[test]
fn builder_with_file_logging() {
    // Test file logging to platform default directory
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .with_file_logging()
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_target_log_dir() {
    // Test with explicit LogDir target
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .target(Target::LogDir { file_name: None })
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_target_custom_folder() {
    // Test with custom folder target
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .target(Target::Folder {
            path: PathBuf::from("/tmp/test-logs"),
            file_name: None,
        })
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_targets_replace() {
    // Test replacing default targets
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .targets([Target::Stderr, Target::Webview])
        .build::<tauri::Wry>();
}

#[test]
fn builder_clear_targets() {
    // Test clearing and rebuilding targets
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .clear_targets()
        .target(Target::Webview)
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_custom_file_name() {
    // Test file logging with custom file name prefix
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .target(Target::LogDir {
            file_name: Some("my-app".to_string()),
        })
        .build::<tauri::Wry>();
}

#[test]
fn builder_full_configuration_with_file_logging() {
    // Test a fully configured builder with file logging
    let _plugin = Builder::new()
        .with_colors()
        .with_max_level(LevelFilter::TRACE)
        .with_target("tao::platform_impl", LevelFilter::WARN)
        .with_target("wry", LevelFilter::WARN)
        .with_file_logging()
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_rotation() {
    // Test different rotation periods
    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation(Rotation::Daily)
        .build::<tauri::Wry>();

    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation(Rotation::Hourly)
        .build::<tauri::Wry>();

    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation(Rotation::Minutely)
        .build::<tauri::Wry>();

    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation(Rotation::Never)
        .build::<tauri::Wry>();
}

#[test]
fn builder_with_rotation_strategy() {
    // Test different rotation strategies
    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation_strategy(RotationStrategy::KeepAll)
        .build::<tauri::Wry>();

    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation_strategy(RotationStrategy::KeepOne)
        .build::<tauri::Wry>();

    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation_strategy(RotationStrategy::KeepSome(7))
        .build::<tauri::Wry>();
}

#[test]
fn builder_full_rotation_configuration() {
    // Test full configuration
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::TRACE)
        .with_target("tao::platform_impl", LevelFilter::WARN)
        .with_file_logging()
        .with_rotation(Rotation::Daily)
        .with_rotation_strategy(RotationStrategy::KeepSome(7))
        .build::<tauri::Wry>();
}

#[test]
fn builder_configured_targets() {
    // Test querying configured targets
    let builder = Builder::new().target(Target::LogDir { file_name: None });

    let targets = builder.configured_targets();
    assert_eq!(targets.len(), 3); // Stdout, Webview (defaults) + LogDir
}

#[test]
fn builder_configured_rotation() {
    // Test querying configured rotation
    let builder = Builder::new().with_rotation(Rotation::Hourly);

    assert!(matches!(builder.configured_rotation(), Rotation::Hourly));
}

#[test]
fn builder_configured_rotation_strategy() {
    // Test querying configured rotation strategy
    let builder = Builder::new().with_rotation_strategy(RotationStrategy::KeepSome(5));

    assert!(matches!(
        builder.configured_rotation_strategy(),
        RotationStrategy::KeepSome(5)
    ));
}

#[test]
fn builder_build_filter() {
    // Test that build_filter returns a valid Targets filter
    let builder = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .with_target("hyper", LevelFilter::WARN);

    let _filter = builder.build_filter();
    // Filter is valid if we get here without panic
}

#[test]
fn builder_build_filter_preserves_builder() {
    // Test that build_filter doesn't consume the builder
    let builder = Builder::new().with_max_level(LevelFilter::DEBUG);

    let _filter1 = builder.build_filter();
    let _filter2 = builder.build_filter(); // Should still work
    let _plugin = builder.build::<tauri::Wry>(); // And this too
}

#[test]
fn builder_without_default_subscriber() {
    // Test that NOT calling with_default_subscriber() still produces a valid plugin
    // This is the recommended pattern - users compose their own subscriber
    let _plugin = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .with_target("hyper", LevelFilter::WARN)
        .build::<tauri::Wry>();
}

#[test]
fn record_payload_serialization() {
    use tauri_plugin_tracing::RecordPayload;

    let payload = RecordPayload {
        message: "test message".to_string(),
        level: LogLevel::Info,
    };

    // Verify payload can be serialized (required for emit)
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("test message"));
    assert!(json.contains("3")); // Info = 3
}

#[test]
fn record_payload_levels() {
    use tauri_plugin_tracing::RecordPayload;

    // Test all log levels serialize correctly
    for (level, expected_num) in [
        (LogLevel::Trace, "1"),
        (LogLevel::Debug, "2"),
        (LogLevel::Info, "3"),
        (LogLevel::Warn, "4"),
        (LogLevel::Error, "5"),
    ] {
        let payload = RecordPayload {
            message: "test".to_string(),
            level,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(
            json.contains(expected_num),
            "Expected {} in {}",
            expected_num,
            json
        );
    }
}

#[test]
fn max_file_size_bytes() {
    let size = MaxFileSize::bytes(1024);
    assert_eq!(size.as_bytes(), 1024);
}

#[test]
fn max_file_size_kb() {
    let size = MaxFileSize::kb(1);
    assert_eq!(size.as_bytes(), 1024);

    let size = MaxFileSize::kb(10);
    assert_eq!(size.as_bytes(), 10 * 1024);
}

#[test]
fn max_file_size_mb() {
    let size = MaxFileSize::mb(1);
    assert_eq!(size.as_bytes(), 1024 * 1024);

    let size = MaxFileSize::mb(10);
    assert_eq!(size.as_bytes(), 10 * 1024 * 1024);
}

#[test]
fn max_file_size_gb() {
    let size = MaxFileSize::gb(1);
    assert_eq!(size.as_bytes(), 1024 * 1024 * 1024);
}

#[test]
fn max_file_size_from_u64() {
    let size: MaxFileSize = 2048u64.into();
    assert_eq!(size.as_bytes(), 2048);
}

#[test]
fn builder_with_max_file_size() {
    let _plugin = Builder::new()
        .with_file_logging()
        .with_max_file_size(MaxFileSize::mb(10))
        .build::<tauri::Wry>();
}

#[test]
fn builder_configured_max_file_size() {
    // Default is None
    let builder = Builder::new();
    assert!(builder.configured_max_file_size().is_none());

    // After setting, should return the configured value
    let builder = Builder::new().with_max_file_size(MaxFileSize::mb(5));
    let max_size = builder.configured_max_file_size().unwrap();
    assert_eq!(max_size.as_bytes(), 5 * 1024 * 1024);
}

#[test]
fn builder_with_max_file_size_and_rotation() {
    // Test combining size-based and time-based rotation
    let _plugin = Builder::new()
        .with_file_logging()
        .with_rotation(Rotation::Daily)
        .with_max_file_size(MaxFileSize::mb(100))
        .with_rotation_strategy(RotationStrategy::KeepSome(7))
        .build::<tauri::Wry>();
}
