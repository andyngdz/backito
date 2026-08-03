//! `backito init`: write the starter config and report what changed.

use super::super::dto::CommandReport;
use super::super::{CliError, ExitStatus};
use crate::features::init::{Overwrite, run_init, summarise};

/// Initialises the working directory.
pub fn run(overwrite: Overwrite) -> Result<CommandReport, CliError> {
    let directory =
        std::env::current_dir().map_err(|source| CliError::WorkingDirectory { source })?;
    let outcome = run_init(&directory, overwrite)?;

    Ok(CommandReport::lines(
        summarise(&outcome),
        ExitStatus::Success,
    ))
}

#[cfg(test)]
#[path = "init_test.rs"]
mod init_test;
