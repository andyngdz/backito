//! Sets a project up: write the config, keep it out of git.

use std::path::Path;

use super::super::{InitError, InitOutcome};
use super::gitignore::ensure_ignored;
use super::write_config::{CONFIG_FILENAME, Overwrite, write_config};

/// Writes `backito.toml` into `directory` and ignores it.
///
/// Both steps run every time. Ignoring is not conditional on having just
/// created the file, so a project whose config predates this command still ends
/// up covered.
pub fn run_init(directory: &Path, overwrite: Overwrite) -> Result<InitOutcome, InitError> {
    let config_path = write_config(directory, overwrite)?;
    let ignore = ensure_ignored(directory, CONFIG_FILENAME)?;

    Ok(InitOutcome {
        config_path,
        ignore,
    })
}

#[cfg(test)]
#[path = "run_init_test.rs"]
mod run_init_test;
