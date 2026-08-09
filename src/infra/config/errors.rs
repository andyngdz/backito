//! Failures raised while loading configuration.

use std::path::PathBuf;

use crate::domain::IntervalError;
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

    /// The storage endpoint is in neither the config nor the environment.
    #[error("[storage] endpoint is unset: put it in the config or set BACKITO_ENDPOINT")]
    MissingEndpoint,

    /// An interval in `[schedule]` could not be read.
    #[error("read {field}: {source}")]
    ParseInterval {
        /// Field the interval was written in.
        field: String,
        /// Why the text is not an interval.
        source: IntervalError,
    },

    /// `[database]` names both a container and a service.
    #[error(
        "[database] sets both container and service: keep container to pin one by name, or service to resolve it by label"
    )]
    ContainerOverSpecified,

    /// `[database]` names neither a container nor a service.
    #[error("[database] needs either container or service to say which container runs Postgres")]
    ContainerUnspecified,
}
