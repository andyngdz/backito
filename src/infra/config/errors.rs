//! Failures raised while loading configuration.

use std::path::PathBuf;
use thiserror::Error;

/// Why configuration could not be loaded.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("read config {path}: {source}")]
    ReadFile {
        /// Path that was attempted.
        path: PathBuf,
        /// Underlying io failure.
        source: std::io::Error,
    },

    /// The config file is not valid TOML, or is missing a required field.
    #[error("parse config {path}: {source}")]
    ParseFile {
        /// Path that was attempted.
        path: PathBuf,
        /// Underlying parse failure.
        source: toml::de::Error,
    },

    /// A credential variable is unset or blank.
    #[error("environment variable {variable} is required and must not be empty")]
    MissingCredential {
        /// Name of the missing variable.
        variable: String,
    },
}
