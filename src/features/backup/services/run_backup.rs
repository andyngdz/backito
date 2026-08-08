//! Takes one backup, in four named phases.

use std::path::Path;
use std::sync::Arc;

use super::super::{BackupError, BackupOutcome};
use super::produce_archive::produce_archive;
use super::publish_archive::publish_archive;
use crate::domain::ArchiveName;
use crate::features::container::resolve;
use crate::features::progress::{ProgressObserver, Step};
use crate::infra::config::{DatabaseSettings, Settings};
use crate::infra::docker::{PostgresTarget, require_running};
use crate::infra::object_store::ObjectStore;

/// Runs a backup end to end and reports what landed in the bucket.
///
/// `utc_stamp` is passed in rather than read from the clock so a caller can make
/// a run reproducible, and so the archive key is decided in one place.
pub async fn run_backup(
    settings: &Settings,
    store: &ObjectStore,
    working_dir: &Path,
    utc_stamp: &str,
    observer: Arc<dyn ProgressObserver>,
) -> Result<BackupOutcome, BackupError> {
    let container = resolve(&settings.database.container).await?;
    let target = target_for(&settings.database, container);
    check_reachable(&target, store, &observer).await?;

    let archive = ArchiveName::new(&settings.database.label, utc_stamp);
    let archive_path = working_dir.join(archive.as_str());

    let produced =
        produce_archive(&target, &settings.database.image, &archive_path, &observer).await?;
    let published =
        publish_archive(store, &archive, &archive_path, produced.bytes, &observer).await?;

    Ok(BackupOutcome {
        archive,
        digest: published.digest,
        local_bytes: produced.bytes,
        stored_bytes: published.stored_bytes,
        tables: produced.tables,
        local_path: archive_path,
    })
}

/// Builds the connection target from configuration and an already-resolved
/// container name.
///
/// The name is a parameter rather than read out of `database`, because the
/// config may hold a service to look up instead of a fixed name and the lookup
/// is a call to Docker. Resolving at the caller is also what lets a long-running
/// caller resolve again on each pass, which is the reason to name a service at
/// all.
pub fn target_for(database: &DatabaseSettings, container: String) -> PostgresTarget {
    PostgresTarget {
        container,
        database: database.name.clone(),
        user: database.user.clone(),
    }
}

/// Proves both ends answer before any long work starts, so a wrong container
/// name or a bad credential costs a second rather than a full dump.
async fn check_reachable(
    target: &PostgresTarget,
    store: &ObjectStore,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<(), BackupError> {
    observer.step_started(Step::CheckDatabase);
    require_running(&target.container).await?;
    observer.step_finished(Step::CheckDatabase, &target.container);

    observer.step_started(Step::CheckStorage);
    let objects = store.list_keys().await?.len();
    observer.step_finished(
        Step::CheckStorage,
        &format!("{} ({objects} objects)", store.bucket_name()),
    );

    Ok(())
}

#[cfg(test)]
#[path = "run_backup_test.rs"]
mod run_backup_test;
