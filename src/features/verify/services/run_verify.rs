//! Restores an archive into a scratch database and compares it to the source.

use std::path::Path;
use std::sync::Arc;

use super::super::{VerifyError, VerifyOutcome};
use super::fetch_archive::fetch_archive;
use super::scratch::{ScratchDatabase, leftover_exists};
use crate::domain::{ArchiveName, compare_counts, rows_behind};
use crate::features::progress::{ProgressObserver, Step};
use crate::infra::config::Settings;
use crate::infra::docker::{
    ARCHIVE_IN_CONTAINER, PostgresTarget, copy_into, restore_in_container, table_counts,
};
use crate::infra::object_store::ObjectStore;

/// Schema whose tables are compared. Only application data is checked: the
/// system schemas a managed image owns differ by design after a restore.
const COMPARED_SCHEMA: &str = "public";

/// Verifies `archive`, or the newest archive when `archive` is `None`.
pub async fn run_verify(
    settings: &Settings,
    store: &ObjectStore,
    source_target: &PostgresTarget,
    working_dir: &Path,
    archive: Option<ArchiveName>,
    observer: Arc<dyn ProgressObserver>,
) -> Result<VerifyOutcome, VerifyError> {
    if leftover_exists(&settings.database.label).await? {
        observer.warn("removing a scratch database left by an earlier run");
    }

    let archive = match archive {
        Some(named) => named,
        None => store.latest_archive(&settings.database.label).await?,
    };
    let archive_path = working_dir.join(archive.as_str());
    let checksum = fetch_archive(store, &archive, &archive_path, &observer).await?;

    let scratch = start_scratch(settings, &observer).await?;
    let restore_errors = load_archive(
        &scratch,
        &archive_path,
        settings.database.restore_jobs,
        &observer,
    )
    .await?;
    let comparisons = compare(source_target, &scratch, &observer).await?;

    observer.step_started(Step::Cleanup);
    scratch.destroy().await?;
    observer.step_finished(Step::Cleanup, scratch.name());

    Ok(VerifyOutcome {
        archive,
        rows_behind: rows_behind(&comparisons),
        comparisons,
        restore_errors,
        checksum,
    })
}

/// Brings up the scratch database.
async fn start_scratch(
    settings: &Settings,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<ScratchDatabase, VerifyError> {
    observer.step_started(Step::StartScratch);
    let scratch =
        ScratchDatabase::start(&settings.database.label, &settings.database.image).await?;
    observer.step_finished(Step::StartScratch, scratch.name());
    Ok(scratch)
}

/// Restores the archive, returning how many errors `pg_restore` reported.
async fn load_archive(
    scratch: &ScratchDatabase,
    archive_path: &Path,
    jobs: u8,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<usize, VerifyError> {
    observer.step_started(Step::Restore);
    let target = scratch.target();
    copy_into(&target.container, archive_path, ARCHIVE_IN_CONTAINER).await?;
    let stderr = restore_in_container(&target, ARCHIVE_IN_CONTAINER, jobs).await?;
    let errors = count_restore_errors(&stderr);
    observer.step_finished(Step::Restore, &format!("{errors} pg_restore errors"));
    Ok(errors)
}

/// Counts rows on both sides and compares them.
async fn compare(
    source_target: &PostgresTarget,
    scratch: &ScratchDatabase,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<Vec<crate::domain::TableComparison>, VerifyError> {
    observer.step_started(Step::CompareRows);
    let source = table_counts(source_target, COMPARED_SCHEMA).await?;
    let restored = table_counts(&scratch.target(), COMPARED_SCHEMA).await?;
    let comparisons = compare_counts(&source, &restored);
    observer.step_finished(Step::CompareRows, &format!("{} tables", comparisons.len()));
    Ok(comparisons)
}

/// Counts the errors `pg_restore` reported.
///
/// Reported for transparency only. Restoring into a managed Postgres image
/// always produces errors for system objects the image already owns, so this
/// number never decides pass or fail.
pub fn count_restore_errors(stderr: &str) -> usize {
    stderr
        .lines()
        .filter(|line| line.contains("error:") || line.contains("ERROR:"))
        .count()
}

#[cfg(test)]
#[path = "run_verify_test.rs"]
mod run_verify_test;
