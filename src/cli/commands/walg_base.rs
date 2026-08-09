//! `backito walg base`: take base backups on a cadence until stopped.

use std::sync::Arc;

use super::super::CliError;
use super::super::dto::CommandReport;
use crate::features::progress::ProgressObserver;
use crate::features::walg::{WalgError, run_base_loop};
use crate::infra::config::Settings;

/// Runs the base backup loop.
///
/// A missing `[walg]` table fails here rather than sleeping forever, which is
/// the opposite of what `archive` does with the same absence. The difference is
/// deliberate: Postgres calls `archive` and reads a failure as "keep the
/// segment", while this is a service someone started on purpose, and a service
/// that cannot do its job should say so instead of looking healthy.
pub async fn run(
    settings: &Settings,
    observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    let Some((walg, credentials)) = settings.walg_runtime() else {
        return Err(CliError::Walg(WalgError::NotConfigured));
    };

    observer.info(&format!(
        "base backup every {}, retaining {} full backups",
        walg.base_interval, walg.retain_full,
    ));

    run_base_loop(walg, credentials, observer).await?;

    Ok(CommandReport::line("base backup loop stopped"))
}

#[cfg(test)]
#[path = "walg_base_test.rs"]
mod walg_base_test;
