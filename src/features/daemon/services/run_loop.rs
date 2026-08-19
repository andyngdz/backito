//! The long-running half of `daemon`: when to act, and what to do about a
//! failure.

use std::path::Path;
use std::sync::Arc;

use super::super::DaemonError;
use super::run_daemon::{PassOutcome, run_pass};
use crate::domain::Interval;
use crate::features::backup::target_for;
use crate::features::container::resolve;
use crate::features::progress::{ProgressObserver, human_bytes};
use crate::features::verify::run_verify;
use crate::infra::config::Settings;
use crate::infra::object_store::ObjectStore;
use crate::infra::workspace::Workspace;

/// How long to wait before retrying after a failed pass.
///
/// Short enough that a transient failure costs one retry rather than a whole
/// cadence, long enough that a persistent one does not spin.
const RETRY_AFTER: Interval = Interval::from_secs(15 * 60);

/// Schedules backups until the process is stopped.
///
/// A failed pass is logged and retried; it does not end the loop. A scheduler
/// that exits on its first failure stops backing up entirely at the moment
/// something goes wrong, which is the moment backups matter most.
pub async fn run_loop(
    settings: &Settings,
    store: &ObjectStore,
    observer: Arc<dyn ProgressObserver>,
) -> Result<(), DaemonError> {
    let mut since_verify = Interval::from_secs(0);

    loop {
        // A fresh workspace per pass, so the pass's multi-GB dump is freed the
        // moment the pass is done instead of surviving the whole daemon life.
        // Acquiring also reclaims any dead workspace a killed earlier run left,
        // so a crash-restart cleans up before it backs up. A workspace that
        // cannot even be created is a full disk, so treat it like any other
        // failed pass and retry rather than exit.
        let workspace = match Workspace::acquire("backito-daemon-") {
            Ok(dir) => dir,
            Err(failure) => {
                observer.warn(&format!(
                    "could not create a workspace, retrying in {RETRY_AFTER}: {failure}"
                ));
                tokio::time::sleep(RETRY_AFTER.as_duration()).await;
                continue;
            }
        };
        let working_dir = workspace.path();

        let wait_for = backup_pass(settings, store, working_dir, &observer).await;

        since_verify = Interval::from_secs(since_verify.as_secs() + wait_for.as_secs());
        if verify_is_due(settings.schedule.verify_interval, since_verify) {
            verify_once(settings, store, working_dir, Arc::clone(&observer)).await;
            since_verify = Interval::from_secs(0);
        }

        // Free this pass's dump before sleeping the interval; the sleep can be a
        // full day and the guard would otherwise hold the dump on disk all day.
        drop(workspace);
        tokio::time::sleep(wait_for.as_duration()).await;
    }
}

/// Runs one backup pass and reports how long to wait before the next one.
///
/// A failed pass is logged and turned into a short retry interval rather than
/// propagated: a scheduler that exits on its first failure stops backing up at
/// the moment something breaks, which is the moment backups matter most.
async fn backup_pass(
    settings: &Settings,
    store: &ObjectStore,
    working_dir: &Path,
    observer: &Arc<dyn ProgressObserver>,
) -> Interval {
    match run_pass(settings, store, working_dir, Arc::clone(observer)).await {
        Ok(PassOutcome::Deferred { remaining }) => {
            observer.info(&format!("a recent archive covers the next {remaining}"));
            remaining
        }
        Ok(PassOutcome::BackedUp { stored, deleted }) => {
            observer.info(&format!("stored {stored}, deleted {deleted} old archives"));
            settings.schedule.backup_interval
        }
        Ok(PassOutcome::Truncated {
            stored,
            local_bytes,
            stored_bytes,
        }) => {
            observer.warn(&format!(
                "{stored} uploaded as {} but was {} on disk, so it may not restore; \
                 kept every older archive and skipped retention",
                human_bytes(stored_bytes),
                human_bytes(local_bytes)
            ));
            RETRY_AFTER
        }
        Err(failure) => {
            observer.warn(&format!(
                "backup pass failed, retrying in {RETRY_AFTER}: {failure}"
            ));
            RETRY_AFTER
        }
    }
}

/// True when verification is enabled and its cadence has come round.
fn verify_is_due(cadence: Interval, elapsed: Interval) -> bool {
    !cadence.is_disabled() && elapsed >= cadence
}

/// Runs one verification, reporting rather than propagating.
///
/// A verification that could not run, and one that ran and found a mismatch,
/// are both news for the operator rather than reasons to stop taking backups.
async fn verify_once(
    settings: &Settings,
    store: &ObjectStore,
    working_dir: &Path,
    observer: Arc<dyn ProgressObserver>,
) {
    let container = match resolve(&settings.database.container).await {
        Ok(name) => name,
        Err(failure) => {
            observer.warn(&format!(
                "skipping verify, no database container: {failure}"
            ));
            return;
        }
    };

    let target = target_for(&settings.database, container);
    match run_verify(
        settings,
        store,
        &target,
        working_dir,
        None,
        observer.clone(),
    )
    .await
    {
        Ok(outcome) if outcome.passed() => observer.info("verify passed"),
        Ok(_) => observer.warn("verify ran and the restored copy did not match the source"),
        Err(failure) => observer.warn(&format!("verify could not run: {failure}")),
    }
}

#[cfg(test)]
#[path = "run_loop_test.rs"]
mod run_loop_test;
