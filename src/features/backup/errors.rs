//! Failures raised while taking a backup.

use thiserror::Error;

use crate::infra::docker::DockerError;
use crate::infra::object_store::ObjectStoreError;

/// Why a backup could not be produced or stored.
#[derive(Debug, Error)]
pub enum BackupError {
    /// The source database could not be reached or dumped.
    #[error("{0}")]
    Database(#[from] DockerError),

    /// The archive could not be stored.
    #[error("{0}")]
    Storage(#[from] ObjectStoreError),

    /// A local file could not be written or read.
    #[error("{operation} {path}: {source}")]
    LocalFile {
        /// What was being attempted.
        operation: String,
        /// Path involved.
        path: String,
        /// Underlying io failure.
        source: std::io::Error,
    },

    /// The archive holds fewer tables than a real backup should.
    ///
    /// Catches a dump that was cut short, or one pointed at an empty database,
    /// before it is uploaded and mistaken for a good backup.
    #[error(
        "archive lists only {found} tables with data, expected at least {expected} -- \
         the dump looks truncated, so nothing was uploaded"
    )]
    TooFewTables {
        /// Tables the archive actually carries.
        found: usize,
        /// Minimum this run required.
        expected: usize,
    },
}
