//! `backito verify`: restore into a throwaway database and compare.

use std::sync::Arc;

use super::super::dto::CommandReport;
use super::super::{CliError, ExitStatus};
use crate::domain::ArchiveName;
use crate::features::backup::target_for;
use crate::features::container::resolve;
use crate::features::progress::ProgressObserver;
use crate::features::verify::{VerifyError, run_verify, summarise};
use crate::infra::config::Settings;
use crate::infra::object_store::ObjectStore;
use crate::infra::workspace::Workspace;

/// Verifies an archive and returns what was found.
pub async fn run(
    settings: &Settings,
    archive: Option<String>,
    observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    let store =
        ObjectStore::new(&settings.storage, &settings.credentials).map_err(VerifyError::Storage)?;

    let workspace = Workspace::acquire("backito-verify-")
        .map_err(|source| CliError::WorkingDirectory { source })?;

    let container = resolve(&settings.database.container).await?;

    let outcome = run_verify(
        settings,
        &store,
        &target_for(&settings.database, container),
        workspace.path(),
        archive.map(ArchiveName::from_key),
        observer,
    )
    .await?;

    let status = if outcome.passed() {
        ExitStatus::Success
    } else {
        ExitStatus::Mismatch
    };
    Ok(CommandReport::lines(summarise(&outcome), status))
}

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;
