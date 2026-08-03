//! Writes the starter config a project fills in.

use std::path::{Path, PathBuf};

use super::super::{FileOperation, InitError};

/// The template shipped in the binary.
///
/// Embedded from the checked-in example so the file a user gets and the file
/// they read in the repository can never drift apart. Anchored at the crate
/// root rather than a `..` hop, which breaks when this module moves.
const CONFIG_TEMPLATE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/backito.example.toml"));

/// Name of the config file a project keeps.
pub const CONFIG_FILENAME: &str = "backito.toml";

/// Whether an existing config may be replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    /// Refuse if the file is already there.
    Refuse,
    /// Replace whatever is there.
    Allow,
}

impl From<bool> for Overwrite {
    /// Maps the `--force` flag onto the permission it grants.
    fn from(force: bool) -> Self {
        match force {
            true => Self::Allow,
            false => Self::Refuse,
        }
    }
}

/// Writes the template into `directory`, returning where it landed.
pub fn write_config(directory: &Path, overwrite: Overwrite) -> Result<PathBuf, InitError> {
    let path = directory.join(CONFIG_FILENAME);

    if overwrite == Overwrite::Refuse && path.exists() {
        return Err(InitError::ConfigExists { path });
    }

    std::fs::write(&path, CONFIG_TEMPLATE).map_err(|source| InitError::File {
        operation: FileOperation::Write,
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

/// The template as it will be written, so a test can assert what ships.
#[cfg(test)]
pub fn template() -> &'static str {
    CONFIG_TEMPLATE
}

#[cfg(test)]
#[path = "write_config_test.rs"]
mod write_config_test;
