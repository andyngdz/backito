//! `backito walg archive`: push one WAL segment, as Postgres asks.

use super::super::CliError;
use super::super::dto::CommandReport;
use crate::features::walg::exec_walg;
use crate::infra::config::Settings;

/// Hands one segment to `wal-g wal-push`.
///
/// Postgres runs this as `archive_command`, once per completed segment, and
/// treats a non-zero exit as "not archived, try again". A missing `[walg]`
/// table exits 0 rather than failing: a development container with no WAL
/// storage should recycle its segments normally instead of filling the disk
/// with unarchivable WAL.
pub fn run(settings: &Settings, segment: &str) -> Result<CommandReport, CliError> {
    let Some((walg, credentials)) = settings.walg_runtime() else {
        return Ok(CommandReport::line("wal archiving is off, segment skipped"));
    };

    // Returns only if the handover failed; a successful exec never comes back.
    Err(CliError::Walg(exec_walg(
        walg,
        credentials,
        &["wal-push", segment],
    )))
}

#[cfg(test)]
#[path = "walg_archive_test.rs"]
mod walg_archive_test;
