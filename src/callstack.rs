//! Call stack parsing and filtering utilities.
//!
//! This module provides types for parsing JavaScript call stacks and extracting
//! meaningful location information for log messages.

use serde::{Deserialize, Serialize};

#[cfg(feature = "colored")]
use colored::*;

/// A single line from a JavaScript call stack.
///
/// This type wraps a string and provides methods for extracting location
/// information while filtering out noise like `node_modules` paths.
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CallStackLine(String);

impl std::ops::Deref for CallStackLine {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for CallStackLine {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<Option<&str>> for CallStackLine {
    fn from(value: Option<&str>) -> Self {
        Self(value.unwrap_or("unknown").to_string())
    }
}

impl Default for CallStackLine {
    fn default() -> Self {
        Self("unknown".to_string())
    }
}

impl std::ops::DerefMut for CallStackLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for CallStackLine {
    #[cfg(feature = "colored")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dimmed())
    }
    #[cfg(not(feature = "colored"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for CallStackLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl CallStackLine {
    /// Replaces occurrences of a substring with another string.
    pub fn replace(&self, from: &str, to: &str) -> Self {
        CallStackLine(self.0.replace(from, to))
    }

    /// Removes the `localhost:PORT/` prefix from URLs for cleaner output.
    fn strip_localhost(&self) -> String {
        let mut result = self.to_string();
        if let Some(start) = result.find("localhost:")
            && let Some(slash_pos) = result[start..].find('/')
        {
            result.replace_range(0..start + slash_pos + 1, "");
        }
        result
    }
}

/// A parsed JavaScript call stack.
///
/// This type parses a newline-separated call stack string and provides methods
/// to extract different levels of location detail for log messages.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CallStack(pub Vec<CallStackLine>);

impl From<Option<&str>> for CallStack {
    fn from(value: Option<&str>) -> Self {
        let lines = value
            .unwrap_or("")
            .split("\n")
            .map(|line| CallStackLine(line.to_string()))
            .collect();
        Self(lines)
    }
}

impl From<Option<String>> for CallStack {
    fn from(value: Option<String>) -> Self {
        let lines = value
            .unwrap_or("".to_string())
            .split("\n")
            .map(|line| CallStackLine(line.to_string()))
            .collect();
        Self(lines)
    }
}

impl CallStack {
    /// Creates a new `CallStack` from an optional string.
    pub fn new(value: Option<&str>) -> Self {
        CallStack::from(value)
    }

    /// Returns the full filtered location as a `#`-separated string.
    ///
    /// This includes all stack frames that pass the filter (excluding
    /// `node_modules` and native code), joined with `#`.
    /// Used for `trace` and `error` log levels.
    pub fn location(&self) -> CallStackLine {
        CallStackLine(
            self.0
                .iter()
                .filter_map(fmap_location)
                .collect::<Vec<String>>()
                .clone()
                .join("#"),
        )
    }

    /// Returns the path of the last (most recent) stack frame.
    ///
    /// This extracts just the last location from the full call stack.
    /// Used for `debug` and `warn` log levels.
    pub fn path(&self) -> CallStackLine {
        match self.location().split("#").last() {
            Some(file_name) => CallStackLine(file_name.to_string()),
            None => CallStackLine("unknown".to_string()),
        }
    }

    /// Returns just the filename (without path) of the most recent stack frame.
    ///
    /// This is the most concise location format.
    /// Used for `info` log level.
    pub fn file_name(&self) -> CallStackLine {
        match self.location().split("/").last() {
            Some(file_name) => CallStackLine(file_name.to_string()),
            None => CallStackLine("unknown".to_string()),
        }
    }
}

/// Substrings that indicate a stack frame should be filtered out.
const FILTERED_LINES: [&str; 2] = ["node_modules", "forEach@[native code]"];

/// Filters and transforms a call stack line.
///
/// Returns `None` if the line should be filtered out (e.g., `node_modules`),
/// otherwise returns the line with localhost URLs stripped.
fn fmap_location(line: &CallStackLine) -> Option<String> {
    if FILTERED_LINES
        .iter()
        .any(|filtered| line.contains(filtered))
    {
        return None;
    }
    Some(line.strip_localhost())
}
