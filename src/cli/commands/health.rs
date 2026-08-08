//! `backito health`: is there a recent enough backup? Exit code says so.

use std::sync::Arc;

use super::super::CliError;
use super::super::dto::CommandReport;
use super::super::errors::ExitStatus;
use crate::features::daemon::{BackupFreshness, DaemonError, backup_freshness, newest_archive};
use crate::features::progress::ProgressObserver;
use crate::infra::config::Settings;
use crate::infra::object_store::ObjectStore;

/// Reports how recent the newest backup is.
///
/// Written for a container healthcheck, so the verdict is the exit code and the
/// line on stdout is for a human reading logs afterwards. The bucket is the
/// source of truth rather than a local marker file, which means a restarted or
/// rebuilt container reports the same answer as the one it replaced.
pub async fn run(
    settings: &Settings,
    _observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    let store =
        ObjectStore::new(&settings.storage, &settings.credentials).map_err(DaemonError::Storage)?;

    let newest = newest_archive(&store, &settings.database.label)
        .await
        .map_err(DaemonError::Storage)?;
    let verdict = backup_freshness(
        &newest,
        settings.schedule.backup_interval,
        jiff::Timestamp::now(),
    );

    let (line, status) = match verdict {
        BackupFreshness::Fresh { age } => (format!("last backup {age} ago"), ExitStatus::Success),
        BackupFreshness::Stale { age, budget } => (
            format!("last backup {age} ago, over the {budget} budget"),
            ExitStatus::Failure,
        ),
        BackupFreshness::Unknown => (
            format!("no readable backup for label {}", settings.database.label),
            ExitStatus::Failure,
        ),
    };

    Ok(CommandReport::lines(vec![line], status))
}

#[cfg(test)]
#[path = "health_test.rs"]
mod health_test;
