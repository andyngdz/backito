//! The base backup loop: what `wal-g backup-push` needs, and how often.
//!
//! WAL archiving alone restores nothing. A segment stream has to be replayed
//! onto a base backup taken from the same cluster, so this is the half that
//! makes the other half worth having.

use std::sync::Arc;

use jiff::Timestamp;
use tokio::process::Command;

use super::super::WalgError;
use super::environment::walg_environment;
use crate::features::daemon::{BackupDue, due_from_age};
use crate::features::progress::ProgressObserver;
use crate::features::walg::services::backup_list::newest_base_age;
use crate::infra::config::{WalgCredentials, WalgSettings};

/// The wal-g subcommand that applies retention.
const DELETE_COMMAND: &str = "delete";

/// How long to wait before retrying after a failed push.
const RETRY_AFTER: crate::domain::Interval = crate::domain::Interval::from_secs(15 * 60);

/// Takes base backups on the configured cadence until the process is stopped.
///
/// On start it asks wal-g when the last base backup landed and waits out what
/// is left of the interval. Without that, every container restart takes a full
/// physical copy of the cluster: three redeploys in a quarter of an hour once
/// produced three base backups and pushed the real one out of a `retain_full`
/// of three.
pub async fn run_base_loop(
    settings: &WalgSettings,
    credentials: &WalgCredentials,
    observer: Arc<dyn ProgressObserver>,
) -> Result<(), WalgError> {
    loop {
        let wait_for = match base_due(settings, credentials, Timestamp::now()).await {
            Ok(BackupDue::NotUntil { remaining }) => {
                observer.info(&format!("a recent base backup covers the next {remaining}"));
                remaining
            }
            Ok(BackupDue::Now) => match push_base(settings, credentials, &observer).await {
                Ok(()) => settings.base_interval,
                Err(failure) => {
                    observer.warn(&format!(
                        "base backup failed, retrying in {RETRY_AFTER}: {failure}"
                    ));
                    RETRY_AFTER
                }
            },
            Err(failure) => {
                observer.warn(&format!(
                    "could not read the backup list, retrying in {RETRY_AFTER}: {failure}"
                ));
                RETRY_AFTER
            }
        };

        tokio::time::sleep(wait_for.as_duration()).await;
    }
}

/// Whether a base backup is due, from what wal-g reports.
async fn base_due(
    settings: &WalgSettings,
    credentials: &WalgCredentials,
    now: Timestamp,
) -> Result<BackupDue, WalgError> {
    let listing = run_walg(settings, credentials, &["backup-list"]).await?;

    Ok(due_from_age(
        newest_base_age(&listing, now),
        settings.base_interval,
    ))
}

/// Takes one base backup and applies retention.
async fn push_base(
    settings: &WalgSettings,
    credentials: &WalgCredentials,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<(), WalgError> {
    observer.info("taking a base backup");
    run_walg(settings, credentials, &["backup-push", &settings.data_dir]).await?;

    // Retention runs only after a push landed. Deleting first would drop the
    // oldest copy in exchange for one that does not exist yet.
    let retain = settings.retain_full.to_string();
    observer.info(&format!("base backup complete, retaining {retain}"));
    run_walg(
        settings,
        credentials,
        &[DELETE_COMMAND, "retain", "FULL", &retain, "--confirm"],
    )
    .await?;

    Ok(())
}

/// Runs `wal-g` and returns its stdout.
async fn run_walg(
    settings: &WalgSettings,
    credentials: &WalgCredentials,
    args: &[&str],
) -> Result<String, WalgError> {
    let mut command = Command::new(&settings.binary);
    command.args(args);
    for (name, value) in walg_environment(settings, credentials) {
        command.env(name, value);
    }

    let output = command.output().await.map_err(|source| WalgError::Exec {
        binary: settings.binary.clone(),
        source,
    })?;

    if !output.status.success() {
        return Err(WalgError::Exit {
            operation: args
                .first()
                .unwrap_or(&crate::infra::config::DEFAULT_WALG_BINARY)
                .to_string(),
            status: output.status.to_string(),
        });
    }

    // wal-g writes its table to stdout and its INFO lines to stderr, but not
    // consistently across versions, so both are read and the parser ignores
    // whatever it does not recognise.
    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

#[cfg(test)]
#[path = "run_base_test.rs"]
mod run_base_test;
