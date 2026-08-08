//! Failures raised by the scheduling loop.

use thiserror::Error;

use crate::features::backup::BackupError;
use crate::infra::object_store::ObjectStoreError;

/// Why the daemon could not start, or could not finish a pass.
///
/// The loop itself survives a failed backup: one bad pass is logged and retried
/// on the next tick, because a scheduler that exits on the first failure stops
/// backing up entirely the moment anything goes wrong. Only failures that make
/// every future pass pointless reach here.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// The bucket could not be reached at startup.
    #[error("{0}")]
    Storage(#[from] ObjectStoreError),

    /// The first backup attempt failed while the loop was starting up, before
    /// anything proved the configuration works.
    #[error("{0}")]
    FirstBackup(#[from] BackupError),
}
