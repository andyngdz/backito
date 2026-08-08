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
