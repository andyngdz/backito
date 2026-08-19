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

    /// A required environment variable is unset or blank.
    #[error("environment variable {variable} is required and must not be empty")]
    MissingEnvVar {
        /// Name of the missing variable.
        variable: String,
    },

    /// An environment variable held a value that could not be parsed.
    #[error("environment variable {variable} is not valid: {reason}")]
    InvalidEnvValue {
        /// Name of the variable.
        variable: String,
        /// Why the value could not be used.
        reason: String,
    },

    /// The config archives WAL, but no WAL credentials were supplied.
    #[error(
        "WAL archiving is configured but its credentials are unset: set BACKITO_WALG_ACCESS_KEY_ID and BACKITO_WALG_SECRET_ACCESS_KEY"
    )]
    MissingWalgCredentials,

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

    /// `[schedule]` keeps no archives at all.
    ///
    /// Retention runs immediately after a backup lands, so zero would delete the
    /// archive that pass just wrote along with every older one. No deployment
    /// wants that, and the cost of accepting the typo is the whole bucket.
    #[error(
        "[schedule] retain is 0, which would delete every archive including the one just taken"
    )]
    RetainsNothing,
}
