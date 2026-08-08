//! The scheduling loop: back up on a cadence, prune, verify now and then.

use std::path::Path;
use std::sync::Arc;

use jiff::Timestamp;

use super::super::DaemonError;
use super::due::{BackupDue, NewestArchive, backup_due};
use super::prune::prune_archives;
use crate::domain::Interval;
use crate::features::backup::run_backup;
use crate::features::progress::ProgressObserver;
use crate::infra::config::Settings;
use crate::infra::object_store::{ObjectStore, ObjectStoreError};

/// Format the archive stamp is written in.
const STAMP_FORMAT: &str = "%Y%m%d-%H%M";

/// Runs one pass: back up if due, prune what fell out of retention, verify when
/// the verify cadence has come round.
///
/// Split out of the loop so a test can drive a single pass without a timer, and
/// so a failure in one pass is visibly a value rather than an early return that
/// kills the schedule.
pub async fn run_pass(
    settings: &Settings,
    store: &ObjectStore,
    working_dir: &Path,
    observer: Arc<dyn ProgressObserver>,
) -> Result<PassOutcome, DaemonError> {
    let newest = newest_archive(store, &settings.database.label).await?;
    let verdict = backup_due(&newest, settings.schedule.backup_interval, Timestamp::now());

    if let BackupDue::NotUntil { remaining } = verdict {
        return Ok(PassOutcome::Deferred { remaining });
    }

    let stamp = Timestamp::now().strftime(STAMP_FORMAT).to_string();
    let outcome = run_backup(settings, store, working_dir, &stamp, Arc::clone(&observer)).await?;
    let pruned = prune_archives(store, &settings.database.label, settings.schedule.retain).await?;

    Ok(PassOutcome::BackedUp {
        stored: outcome.archive.to_string(),
        deleted: pruned.deleted.len(),
    })
}

/// What one pass did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassOutcome {
    /// A backup ran and retention was applied.
    BackedUp {
        /// Key the archive landed under.
        stored: String,
        /// Archives deleted to honour retention.
        deleted: usize,
    },
    /// A recent enough archive already exists.
    Deferred {
        /// Time left before the next backup is due.
        remaining: Interval,
    },
}

/// What the bucket holds for `label`.
///
/// An empty bucket is a normal first run rather than a failure, so it becomes
/// `Absent` instead of propagating as an error.
pub async fn newest_archive(
    store: &ObjectStore,
    label: &str,
) -> Result<NewestArchive, ObjectStoreError> {
    match store.latest_archive(label).await {
        Ok(archive) => Ok(match archive.stamp() {
            Some(stamp) => NewestArchive::Stamped(stamp.to_owned()),
            None => NewestArchive::Unstamped,
        }),
        Err(ObjectStoreError::NoArchives { .. }) => Ok(NewestArchive::Absent),
        Err(other) => Err(other),
    }
}

#[cfg(test)]
#[path = "run_daemon_test.rs"]
mod run_daemon_test;
