use serde::{Deserialize, Serialize};

#[cfg(feature = "colored")]
use colored::*;

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
    pub fn replace(&self, from: &str, to: &str) -> Self {
        CallStackLine(self.0.replace(from, to))
    }
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
    pub fn new(value: Option<&str>) -> Self {
        CallStack::from(value)
    }

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

    pub fn path(&self) -> CallStackLine {
        match self.location().split("#").last() {
            Some(file_name) => CallStackLine(file_name.to_string()),
            None => CallStackLine("unknown".to_string()),
        }
    }

    pub fn file_name(&self) -> CallStackLine {
        match self.location().split("/").last() {
            Some(file_name) => CallStackLine(file_name.to_string()),
            None => CallStackLine("unknown".to_string()),
        }
    }
}

const FILTERED_LINES: [&str; 2] = ["node_modules", "forEach@[native code]"];

fn fmap_location(line: &CallStackLine) -> Option<String> {
    if FILTERED_LINES
        .iter()
        .any(|filtered| line.contains(filtered))
    {
        return None;
    }
    Some(line.strip_localhost())
}
