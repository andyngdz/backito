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
use crate::infra::shutdown::{Woke, sleep_unless_stopped, unless_stopped};
use crate::infra::workspace::Workspace;

/// How long to wait before retrying after a failed pass.
///
/// Short enough that a transient failure costs one retry rather than a whole
/// cadence, long enough that a persistent one does not spin.
const RETRY_AFTER: Interval = Interval::from_secs(15 * 60);

/// What the loop should do once a cycle is over.
enum Cycle {
    /// Wait this long, then go round again.
    Again(Interval),
    /// A stop was requested part way through.
    Stop,
}

/// Schedules backups until the process is stopped.
///
/// A failed pass is logged and retried; it does not end the loop. A scheduler
/// that exits on its first failure stops backing up entirely at the moment
/// something goes wrong, which is the moment backups matter most. A stop
/// requested by the operator is the one thing that does end it.
pub async fn run_loop(
    settings: &Settings,
    store: &ObjectStore,
    observer: Arc<dyn ProgressObserver>,
) -> Result<(), DaemonError> {
    let mut since_verify = Interval::from_secs(0);

    loop {
        let waiting = match one_cycle(settings, store, &observer, &mut since_verify).await {
            Cycle::Stop => return stop(&observer),
            Cycle::Again(wait_for) => wait_for,
        };

        if sleep_unless_stopped(waiting.as_duration()).await == Woke::Stopping {
            return stop(&observer);
        }
    }
}

/// Runs one backup pass and, when its cadence has come round, one verification.
///
/// The workspace is scoped to this function so the pass's multi-GB dump is freed
/// before the caller sleeps the interval, which can be a full day.
async fn one_cycle(
    settings: &Settings,
    store: &ObjectStore,
    observer: &Arc<dyn ProgressObserver>,
    since_verify: &mut Interval,
) -> Cycle {
    // A workspace that cannot even be created is a full disk, so treat it like
    // any other failed pass and retry rather than exit. Acquiring also reclaims
    // any dead workspace a killed earlier run left behind.
    let workspace = match Workspace::acquire("backito-daemon-") {
        Ok(dir) => dir,
        Err(failure) => {
            observer.warn(&format!(
                "could not create a workspace, retrying in {RETRY_AFTER}: {failure}"
            ));
            return Cycle::Again(RETRY_AFTER);
        }
    };
    let working_dir = workspace.path();

    // A stop during the pass drops the unfinished work rather than waiting it
    // out: a dump can run for an hour, and an orchestrator that asked to stop
    // will send SIGKILL long before that. Dropping is what runs the guards,
    // which is what removes the scratch container and the dump on disk.
    let Some(wait_for) = unless_stopped(backup_pass(settings, store, working_dir, observer)).await
    else {
        return Cycle::Stop;
    };

    *since_verify = since_verify.saturating_add(wait_for);
    if verify_is_due(settings.schedule.verify_interval, *since_verify) {
        let verify = verify_once(settings, store, working_dir, Arc::clone(observer));
        if unless_stopped(verify).await.is_none() {
            return Cycle::Stop;
        }
        *since_verify = Interval::from_secs(0);
    }

    Cycle::Again(wait_for)
}

/// Reports the stop and ends the loop.
fn stop(observer: &Arc<dyn ProgressObserver>) -> Result<(), DaemonError> {
    observer.info("stopping");
    Ok(())
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
