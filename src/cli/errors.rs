//! Maps command failures onto messages and exit codes.

use thiserror::Error;

use crate::features::backup::BackupError;
use crate::features::restore::RestoreError;
use crate::features::verify::VerifyError;
use crate::infra::config::ConfigError;

/// How a command ended, as the shell sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitStatus {
    /// The command did what was asked.
    Success = 0,
    /// The command could not run to completion.
    Failure = 1,
    /// A verification ran and found a mismatch. Distinct from `Failure` so a
    /// scheduled check can tell "could not verify" from "did not match".
    Mismatch = 2,
}

impl ExitStatus {
    /// The code handed to the process exit.
    pub fn code(self) -> i32 {
        self as i32
    }
}

/// Anything that stops a command.
#[derive(Debug, Error)]
pub enum CliError {
    /// Configuration could not be loaded.
    #[error("{0}")]
    Config(#[from] ConfigError),

    /// A backup failed.
    #[error("{0}")]
    Backup(#[from] BackupError),

    /// A verification could not be carried out.
    #[error("{0}")]
    Verify(#[from] VerifyError),

    /// A restore failed or was refused.
    #[error("{0}")]
    Restore(#[from] RestoreError),

    /// A working directory could not be created.
    #[error("create working directory: {source}")]
    WorkingDirectory {
        /// Underlying io failure.
        source: std::io::Error,
    },
}

impl CliError {
    /// The next thing the user can do, when there is one worth naming.
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Self::Config(ConfigError::ReadFile { .. }) => {
                Some("create backito.toml, or pass --config with its path")
            }
            Self::Config(ConfigError::MissingCredential { .. }) => Some(
                "export BACKITO_ACCESS_KEY_ID and BACKITO_SECRET_ACCESS_KEY, \
                 or run the command through your secret manager",
            ),
            Self::Backup(BackupError::Database(_)) | Self::Restore(RestoreError::Database(_)) => {
                Some("check the container name in backito.toml and that it is running")
            }
            Self::Backup(BackupError::Storage(_)) | Self::Verify(VerifyError::Storage(_)) => {
                Some("check the bucket name, endpoint, and that the credential covers this bucket")
            }
            Self::Config(ConfigError::ParseFile { .. })
            | Self::Backup(_)
            | Self::Verify(_)
            | Self::Restore(_)
            | Self::WorkingDirectory { .. } => None,
        }
    }
}

#[cfg(test)]
#[path = "errors_test.rs"]
mod errors_test;
