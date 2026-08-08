//! `backito walg entrypoint`: configure Postgres for archiving, then hand over.

use std::path::Path;

use super::super::CliError;
use super::super::dto::CommandReport;
use crate::features::walg::{WalgError, archiving_fragment, disabled_fragment, exec_program};
use crate::infra::config::{Settings, WalgMode};

/// Writes the archiving fragment to `fragment_path`, then execs `program`.
///
/// This is a container entrypoint: it replaces itself with the image's real one
/// rather than supervising it, so Postgres keeps PID 1 and signals reach it
/// unchanged.
pub fn run(
    settings: &Settings,
    fragment_path: &Path,
    config_path: &Path,
    program: &str,
    args: &[String],
) -> Result<CommandReport, CliError> {
    let fragment = match &settings.walg {
        WalgMode::Enabled(_) => archiving_fragment(config_path),
        WalgMode::Disabled => disabled_fragment(),
    };

    std::fs::write(fragment_path, fragment).map_err(|source| {
        CliError::Walg(WalgError::WriteConfig {
            path: fragment_path.display().to_string(),
            source,
        })
    })?;

    Err(CliError::Walg(exec_program(program, args)))
}

#[cfg(test)]
#[path = "walg_entrypoint_test.rs"]
mod walg_entrypoint_test;
