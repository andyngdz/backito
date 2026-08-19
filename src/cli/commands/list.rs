//! `backito list`: report which archives the bucket holds.

use super::super::dto::CommandReport;
use super::super::{CliError, ExitStatus};
use crate::features::list::{Detail, render};
use crate::features::verify::VerifyError;
use crate::infra::config::Settings;
use crate::infra::object_store::ObjectStore;

/// Lists the archives stored for the configured label.
pub async fn run(settings: &Settings, detail: Detail) -> Result<CommandReport, CliError> {
    let store =
        ObjectStore::new(&settings.storage, &settings.credentials).map_err(VerifyError::Storage)?;

    let archives = store
        .list_archives(&settings.database.label)
        .await
        .map_err(VerifyError::Storage)?;

    Ok(CommandReport::lines(
        render(&archives, detail),
        ExitStatus::Success,
    ))
}

#[cfg(test)]
#[path = "list_test.rs"]
mod list_test;
