//! `backito walg`: pick the subcommand and hand it the settings it needs.

use super::super::CliError;
use super::super::args::WalgCommand;
use super::super::dto::CommandReport;
use super::{SourceChoice, load, walg_archive, walg_base, walg_entrypoint};

/// Runs one of the `walg` subcommands.
///
/// `entrypoint` is the reason the source choice is threaded through rather than
/// left to the default: the fragment it writes names that source for Postgres to
/// repeat later, from a working directory of Postgres's choosing.
pub async fn run(command: WalgCommand, choice: &SourceChoice) -> Result<CommandReport, CliError> {
    let context = load(choice)?;

    match command {
        WalgCommand::Archive { segment } => walg_archive::run(&context.settings, &segment),

        WalgCommand::Base => walg_base::run(&context.settings, context.observer).await,

        WalgCommand::Entrypoint {
            fragment,
            program,
            args,
        } => walg_entrypoint::run(
            &context.settings,
            &fragment,
            &choice.cli_flags(),
            &program,
            &args,
        ),
    }
}

#[cfg(test)]
#[path = "walg_test.rs"]
mod walg_test;
