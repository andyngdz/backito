//! Loads an archive into a real database, once the guard allows it.

use std::path::Path;
use std::sync::Arc;

use super::super::{RestoreError, RestoreOutcome};
use super::guard::{RestoreAuthorisation, ensure_writable, populated_tables};
use crate::domain::ArchiveName;
use crate::features::progress::{ProgressObserver, Step, human_bytes};
use crate::infra::docker::{
    ARCHIVE_IN_CONTAINER, PostgresTarget, copy_into, require_running, restore_in_container,
    trailing_stderr,
};
use crate::infra::object_store::ObjectStore;

/// What the caller decided about a restore, separate from where the archive
/// lives and which container receives it.
pub struct RestoreRequest {
    /// Archive to restore, or `None` for the newest archive in the bucket.
    pub archive: Option<ArchiveName>,
    /// Whether a target that already holds data may be written to.
    pub authorisation: RestoreAuthorisation,
    /// Parallelism `pg_restore` runs at. Lower it for a memory-capped container
    /// that would otherwise OOM mid-restore.
    pub jobs: u8,
}

/// Restores the requested archive into `target`, or the newest when unset.
pub async fn run_restore(
    store: &ObjectStore,
    label: &str,
    target: &PostgresTarget,
    working_dir: &Path,
    request: RestoreRequest,
    observer: Arc<dyn ProgressObserver>,
) -> Result<RestoreOutcome, RestoreError> {
    observer.step_started(Step::CheckDatabase);
    require_running(&target.container).await?;
    ensure_writable(target, &request.authorisation).await?;
    observer.step_finished(Step::CheckDatabase, &target.container);

    let archive = match request.archive {
        Some(named) => named,
        None => store.latest_archive(label).await?,
    };
    let archive_path = working_dir.join(archive.as_str());

    observer.step_started(Step::Download);
    let bytes = store.object_size(archive.as_str()).await?;
    observer.transfer_started(Some(bytes));
    store.download_file(archive.as_str(), &archive_path).await?;
    observer.transfer_finished();
    observer.step_finished(Step::Download, &human_bytes(bytes));

    observer.step_started(Step::Restore);
    copy_into(&target.container, &archive_path, ARCHIVE_IN_CONTAINER).await?;
    let stderr = restore_in_container(target, ARCHIVE_IN_CONTAINER, request.jobs).await?;

    // `pg_restore`'s exit code is ignored on purpose, so an empty target is the
    // only reliable signal that no rows landed: a container OOM-killed mid-load
    // leaves the schema in place but every table empty, which otherwise reads as
    // success. Refuse it here so the caller never reports a hollow restore.
    if populated_tables(target).await? == 0 {
        return Err(RestoreError::RestoreLoadedNothing {
            container: target.container.clone(),
            database: target.database.clone(),
            stderr: trailing_stderr(stderr.as_bytes()),
        });
    }
    observer.step_finished(Step::Restore, &target.database);

    Ok(RestoreOutcome {
        archive,
        bytes,
        restore_stderr: stderr,
    })
}

#[cfg(test)]
#[path = "run_restore_test.rs"]
mod run_restore_test;
