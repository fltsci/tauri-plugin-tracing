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
                .filter(|line| !line.contains("node_modules") && line.contains("src"))
                .map(|line| line.replace("@http://localhost:1420/", "").to_string())
                .collect::<Vec<String>>()
                .clone()
                .join("#"),
        )
    }

    pub fn file_name(&self) -> CallStackLine {
        match self.location().split("/").last() {
            Some(file_name) => CallStackLine(file_name.to_string()),
            None => CallStackLine("unknown".to_string()),
        }
    }
}
