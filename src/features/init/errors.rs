//! Failures raised while setting a project up.

use std::path::PathBuf;
use thiserror::Error;

/// What was being done to a file when it failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    /// Reading it.
    Read,
    /// Writing it.
    Write,
}

impl FileOperation {
    /// The verb used in the message.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

impl std::fmt::Display for FileOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Why a project could not be initialised.
#[derive(Debug, Error)]
pub enum InitError {
    /// A config file is already there.
    ///
    /// Overwriting it would discard a bucket name and endpoint the user typed
    /// in by hand, so it takes an explicit `--force`.
    #[error("{path} already exists -- pass --force to overwrite it")]
    ConfigExists {
        /// File that was already present.
        path: PathBuf,
    },

    /// A file could not be read or written.
    #[error("{operation} {path}: {source}")]
    File {
        /// What was being attempted.
        operation: FileOperation,
        /// Path involved.
        path: PathBuf,
        /// Underlying io failure.
        source: std::io::Error,
    },
}
