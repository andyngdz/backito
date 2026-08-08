//! `backito walg`: pick the subcommand and hand it the settings it needs.

use std::path::Path;

use super::super::CliError;
use super::super::args::WalgCommand;
use super::super::dto::CommandReport;
use super::{load, walg_archive, walg_base, walg_entrypoint};
use crate::infra::config::CONFIG_FILENAME;

/// Runs one of the `walg` subcommands.
///
/// `entrypoint` is the reason the config path is threaded through rather than
/// left to the default: the fragment it writes names that path for Postgres to
/// use later, from a working directory of Postgres's choosing.
pub async fn run(command: WalgCommand, config: Option<&Path>) -> Result<CommandReport, CliError> {
    let context = load(config)?;

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
            config.unwrap_or(Path::new(CONFIG_FILENAME)),
            &program,
            &args,
        ),
    }
}

#[cfg(test)]
#[path = "walg_test.rs"]
mod walg_test;
