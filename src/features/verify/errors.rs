//! Failures raised while verifying an archive.

use thiserror::Error;

use crate::features::backup::BackupError;
use crate::infra::docker::DockerError;
use crate::infra::object_store::ObjectStoreError;

/// Why a verification could not be completed.
///
/// A verification that runs and finds a mismatch is not an error -- it is a
/// result. This type covers only the cases where the check itself could not be
/// carried out.
#[derive(Debug, Error)]
pub enum VerifyError {
    /// A database or container operation failed.
    #[error("{0}")]
    Database(#[from] DockerError),

    /// The archive could not be fetched.
    #[error("{0}")]
    Storage(#[from] ObjectStoreError),

    /// The downloaded archive could not be hashed.
    #[error("{0}")]
    Checksum(#[from] BackupError),

    /// The scratch container name is already taken by something that is not a
    /// scratch container, so the run refuses to touch it.
    #[error(
        "container {container} already exists and is not a backito scratch container -- \
         remove it or set a different scratch name before verifying"
    )]
    ScratchNameTaken {
        /// Container that blocked the run.
        container: String,
    },
}
