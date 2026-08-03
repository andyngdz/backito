//! Loads an archive into a real database, once the guard allows it.

use humansize::{BINARY, format_size};
use std::path::Path;
use std::sync::Arc;

use super::super::{RestoreError, RestoreOutcome};
use super::guard::{RestoreAuthorisation, ensure_writable};
use crate::domain::ArchiveName;
use crate::features::progress::{ProgressObserver, Step};
use crate::infra::docker::{PostgresTarget, copy_into, require_running, restore_in_container};
use crate::infra::object_store::ObjectStore;

/// Path the archive is copied to inside the target container.
const ARCHIVE_IN_CONTAINER: &str = "/tmp/backito-restore.dump";

/// Parallel jobs `pg_restore` uses.
const RESTORE_JOBS: u8 = 4;

/// Restores `archive` into `target`, or the newest archive when `None`.
pub async fn run_restore(
    store: &ObjectStore,
    label: &str,
    target: &PostgresTarget,
    working_dir: &Path,
    archive: Option<ArchiveName>,
    authorisation: RestoreAuthorisation,
    observer: Arc<dyn ProgressObserver>,
) -> Result<RestoreOutcome, RestoreError> {
    observer.step_started(Step::CheckDatabase);
    require_running(&target.container).await?;
    ensure_writable(target, &authorisation).await?;
    observer.step_finished(Step::CheckDatabase, &target.container);

    let archive = match archive {
        Some(named) => named,
        None => store.latest_archive(label).await?,
    };
    let archive_path = working_dir.join(archive.as_str());

    observer.step_started(Step::Download);
    let bytes = store.object_size(archive.as_str()).await?;
    observer.transfer_started(Some(bytes));
    store.download_file(archive.as_str(), &archive_path).await?;
    observer.transfer_finished();
    observer.step_finished(Step::Download, &format_size(bytes, BINARY));

    observer.step_started(Step::Restore);
    copy_into(&target.container, &archive_path, ARCHIVE_IN_CONTAINER).await?;
    let stderr = restore_in_container(target, ARCHIVE_IN_CONTAINER, RESTORE_JOBS).await?;
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
