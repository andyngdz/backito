//! Maps command failures onto messages and exit codes.

use thiserror::Error;

use crate::features::backup::BackupError;
use crate::features::daemon::DaemonError;
use crate::features::init::InitError;
use crate::features::restore::RestoreError;
use crate::features::verify::VerifyError;
use crate::features::walg::WalgError;
use crate::infra::config::ConfigError;
use crate::infra::docker::DockerError;
use crate::infra::object_store::ObjectStoreError;

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

    /// A project could not be initialised.
    #[error("{0}")]
    Init(#[from] InitError),

    /// The scheduling loop could not start or could not continue.
    #[error("{0}")]
    Daemon(#[from] DaemonError),

    /// A wal-g command could not do its work.
    #[error("{0}")]
    Walg(#[from] WalgError),

    /// The container running the database could not be identified.
    #[error("{0}")]
    Container(#[from] DockerError),

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
                Some("run `backito init` here to write one, or pass --config with its path")
            }
            Self::Init(InitError::ConfigExists { .. }) => {
                Some("edit the existing file, or re-run with --force to replace it")
            }
            Self::Config(ConfigError::MissingCredential { .. }) => Some(
                "export BACKITO_ACCESS_KEY_ID and BACKITO_SECRET_ACCESS_KEY, \
                 or run the command through your secret manager",
            ),
            Self::Backup(BackupError::Database(_)) | Self::Restore(RestoreError::Database(_)) => {
                Some("check the container name in backito.toml and that it is running")
            }
            Self::Backup(BackupError::Storage(failure))
            | Self::Verify(VerifyError::Storage(failure))
            | Self::Restore(RestoreError::Storage(failure)) => Some(Self::storage_hint(failure)),
            Self::Config(
                ConfigError::ContainerOverSpecified | ConfigError::ContainerUnspecified,
            ) => Some("set exactly one of container or service under [database]"),
            Self::Config(ConfigError::ParseInterval { .. }) => {
                Some("write intervals as a number and a unit: 30s, 15m, 24h, 7d")
            }
            Self::Walg(WalgError::NotConfigured) => {
                Some("add a [walg] section with an s3_prefix, or leave WAL archiving off")
            }
            Self::Container(DockerError::NoContainerForService { .. }) => Some(
                "check the service is up, and that container_label matches the orchestrator: \
                 com.docker.compose.service for compose, uncloud.service.name for uncloud",
            ),
            Self::Config(ConfigError::ParseFile { .. })
            | Self::Backup(_)
            | Self::Verify(_)
            | Self::Restore(_)
            | Self::Init(_)
            | Self::Container(_)
            | Self::Daemon(_)
            | Self::Walg(_)
            | Self::WorkingDirectory { .. } => None,
        }
    }

    /// The next step for a store failure.
    ///
    /// A missing key is not a misconfigured bucket: the request was authorised
    /// and answered, so pointing the user at credentials sends them after the
    /// wrong thing.
    fn storage_hint(failure: &ObjectStoreError) -> &'static str {
        if failure.is_missing_object() {
            return "no object at that key -- list the bucket to see which archives exist, \
                  or drop --archive to use the newest";
        }

        match failure {
            ObjectStoreError::Configure { .. }
            | ObjectStoreError::Request { .. }
            | ObjectStoreError::LocalStream { .. }
            | ObjectStoreError::LocalFile { .. }
            | ObjectStoreError::NoArchives { .. } => {
                "check the bucket name, endpoint, and that the credential covers this bucket"
            }
        }
    }
}

#[cfg(test)]
#[path = "errors_test.rs"]
mod errors_test;
