//! `backito restore`: load an archive into a real database.

use std::sync::Arc;

use super::super::CliError;
use super::super::dto::CommandReport;
use crate::domain::ArchiveName;
use crate::features::backup::target_for;
use crate::features::progress::ProgressObserver;
use crate::features::restore::{RestoreAuthorisation, RestoreError, RestoreRequest, run_restore};
use crate::infra::config::Settings;
use crate::infra::object_store::ObjectStore;

/// Restores an archive into the configured database, or into `into_container`.
pub async fn run(
    settings: &Settings,
    into_container: Option<String>,
    archive: Option<String>,
    authorisation: RestoreAuthorisation,
    observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    let store = ObjectStore::new(&settings.storage, &settings.credentials)
        .map_err(RestoreError::Storage)?;

    let mut target = target_for(&settings.database);
    if let Some(container) = into_container {
        target.container = container;
    }

    let workspace = tempfile::Builder::new()
        .prefix("backito-restore-")
        .tempdir()
        .map_err(|source| CliError::WorkingDirectory { source })?;

    let request = RestoreRequest {
        archive: archive.map(ArchiveName::from_key),
        authorisation,
        jobs: settings.database.restore_jobs,
    };

    let outcome = run_restore(
        &store,
        &settings.database.label,
        &target,
        workspace.path(),
        request,
        observer,
    )
    .await?;

    tracing::info!(
        archive = %outcome.archive,
        bytes = outcome.bytes,
        stderr = %outcome.restore_stderr,
        "restore finished"
    );

    Ok(CommandReport::line(outcome.archive.as_str()))
}

#[cfg(test)]
#[path = "restore_test.rs"]
mod restore_test;
