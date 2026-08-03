//! Failures raised while restoring into a real database.

use thiserror::Error;

use crate::infra::docker::DockerError;
use crate::infra::object_store::ObjectStoreError;

/// Why a restore was refused or could not complete.
#[derive(Debug, Error)]
pub enum RestoreError {
    /// A database or container operation failed.
    #[error("{0}")]
    Database(#[from] DockerError),

    /// The archive could not be fetched.
    #[error("{0}")]
    Storage(#[from] ObjectStoreError),

    /// The target already holds data and the caller did not confirm.
    #[error(
        "{container}/{database} already holds {tables} tables with data -- \
         restoring would write into a live database. Re-run with --force to proceed."
    )]
    TargetNotEmpty {
        /// Container that would be written to.
        container: String,
        /// Database that would be written to.
        database: String,
        /// Tables already present.
        tables: usize,
    },
}
