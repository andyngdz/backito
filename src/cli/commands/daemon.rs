//! `backito daemon`: run the schedule until the process is stopped.

use std::sync::Arc;

use super::super::CliError;
use super::super::dto::CommandReport;
use crate::features::daemon::{DaemonError, run_loop};
use crate::features::progress::ProgressObserver;
use crate::infra::config::{ScheduleSettings, Settings};
use crate::infra::object_store::ObjectStore;

/// Runs the scheduling loop.
///
/// Returns only when the loop cannot continue, so the `CommandReport` in the
/// success position is unreachable in practice; it exists because every command
/// returns one.
pub async fn run(
    settings: &Settings,
    observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    let store =
        ObjectStore::new(&settings.storage, &settings.credentials).map_err(DaemonError::Storage)?;

    // Proven once, before the loop starts. Inside the loop a failed pass is
    // logged and retried, which is right for a transient outage and wrong for a
    // bucket name that will never resolve: that would retry in silence forever
    // rather than saying the configuration is broken.
    store.list_keys().await.map_err(DaemonError::Storage)?;

    let workspace = tempfile::Builder::new()
        .prefix("backito-daemon-")
        .tempdir()
        .map_err(|source| CliError::WorkingDirectory { source })?;

    let schedule: &ScheduleSettings = &settings.schedule;
    observer.info(&format!(
        "backup every {}, verify every {}, retaining {} archives",
        schedule.backup_interval, schedule.verify_interval, schedule.retain,
    ));

    run_loop(settings, &store, workspace.path(), observer).await?;

    Ok(CommandReport::line("daemon stopped"))
}

#[cfg(test)]
#[path = "daemon_test.rs"]
mod daemon_test;
