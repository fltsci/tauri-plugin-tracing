//! Error types for the tracing plugin.

use serde::{Serialize, Serializer};

/// A specialized [`Result`](std::result::Result) type for tracing plugin operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the tracing plugin.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Error {
    /// An error from the Tauri runtime.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    Tauri(#[from] tauri::Error),

    /// An I/O error, typically from file operations.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    Io(#[from] std::io::Error),

    /// An error formatting a timestamp.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    TimeFormat(#[from] time::error::Format),

    /// An invalid time format description was provided.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),

    /// The internal logger was not initialized.
    #[error("Internal logger disabled and cannot be acquired or attached")]
    LoggerNotInitialized,

    /// Failed to set the global default subscriber.
    #[error(transparent)]
    #[cfg_attr(feature = "specta", specta(skip))]
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),

    /// The requested feature is not yet implemented.
    #[error("Not implemented")]
    NotImplemented,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
