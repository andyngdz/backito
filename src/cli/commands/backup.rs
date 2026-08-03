//! `backito backup`: map arguments, run the feature, return the result.

use std::sync::Arc;

use super::super::CliError;
use super::super::dto::CommandReport;
use super::super::reporter::human_bytes;
use crate::features::backup::{BackupError, keep_archive, run_backup};
use crate::features::progress::ProgressObserver;
use crate::infra::config::Settings;
use crate::infra::object_store::ObjectStore;

/// What happens to the archive on disk once it is uploaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalCopy {
    /// Keep it in the working directory.
    Keep,
    /// Delete it with the temporary workspace.
    Discard,
}

impl From<bool> for LocalCopy {
    /// Maps the `--keep` flag onto the disposition it names.
    fn from(keep: bool) -> Self {
        match keep {
            true => Self::Keep,
            false => Self::Discard,
        }
    }
}

/// Runs a backup and reports the stored key.
pub async fn run(
    settings: &Settings,
    local_copy: LocalCopy,
    observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    let store =
        ObjectStore::new(&settings.storage, &settings.credentials).map_err(BackupError::Storage)?;

    let workspace = tempfile::Builder::new()
        .prefix("backito-")
        .tempdir()
        .map_err(|source| CliError::WorkingDirectory { source })?;

    let stamp = jiff::Timestamp::now().strftime("%Y%m%d-%H%M").to_string();
    let outcome = run_backup(
        settings,
        &store,
        workspace.path(),
        &stamp,
        Arc::clone(&observer),
    )
    .await?;

    if !outcome.sizes_match() {
        observer.warn(&format!(
            "stored size {} differs from local size {} -- the upload may be truncated",
            human_bytes(outcome.stored_bytes),
            human_bytes(outcome.local_bytes)
        ));
    }

    match local_copy {
        LocalCopy::Keep => {
            let directory =
                std::env::current_dir().map_err(|source| CliError::WorkingDirectory { source })?;
            let kept = keep_archive(&outcome.local_path, &directory, outcome.archive.as_str())?;
            observer.warn(&format!("kept local copy at {}", kept.display()));
        }
        LocalCopy::Discard => {}
    }

    tracing::info!(
        archive = %outcome.archive,
        digest = %outcome.digest,
        tables = outcome.tables,
        "backup stored"
    );

    Ok(CommandReport::line(outcome.archive.as_str()))
}

#[cfg(test)]
#[path = "backup_test.rs"]
mod backup_test;
