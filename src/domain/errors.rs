//! Failures raised while reading a domain value from text.

use thiserror::Error;

/// Why an interval could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IntervalError {
    /// Nothing, or only whitespace.
    #[error("an interval cannot be empty")]
    Empty,

    /// The trailing unit is missing or not one of s/m/h/d.
    #[error("interval {text} needs a unit: s, m, h, or d, as in 24h")]
    UnknownUnit {
        /// The text as written.
        text: String,
    },

    /// The leading number could not be read.
    #[error("interval {text} does not start with a whole number")]
    NotANumber {
        /// The text as written.
        text: String,
    },
}

/// Why an archive key supplied by a person could not be used.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArchiveKeyError {
    /// The key is not one this tool wrote for the configured label.
    ///
    /// Raised before anything downloads, because the key becomes a local
    /// filename: one carrying a path segment would write outside the workspace.
    #[error("--archive {key} is not an archive backito wrote for label {label}")]
    NotOurs {
        /// The key as typed.
        key: String,
        /// Label the config pins.
        label: String,
    },
}
