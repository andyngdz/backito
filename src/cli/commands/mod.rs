//! One file per command: map arguments, call a feature, hand back a report.

mod backup;
mod restore;
mod verify;

use std::sync::Arc;

use super::CliError;
use super::args::Command;
use super::dto::CommandReport;
use crate::features::progress::ProgressObserver;
use crate::infra::config::Settings;

/// Runs the requested command.
pub async fn dispatch(
    command: Command,
    settings: &Settings,
    observer: Arc<dyn ProgressObserver>,
) -> Result<CommandReport, CliError> {
    match command {
        Command::Backup { keep } => backup::run(settings, keep.into(), observer).await,
        Command::Verify { archive } => verify::run(settings, archive, observer).await,
        Command::Restore {
            into_container,
            archive,
            force,
        } => restore::run(settings, into_container, archive, force.into(), observer).await,
    }
}
