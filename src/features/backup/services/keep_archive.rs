//! Copies a finished archive out of the temporary workspace.

use std::path::{Path, PathBuf};

use super::super::BackupError;

/// Copies `source` into `directory` under `name`, returning where it landed.
///
/// The archive is written to a temporary workspace that disappears when the
/// command ends, so keeping a copy means moving it somewhere the user chose.
pub fn keep_archive(source: &Path, directory: &Path, name: &str) -> Result<PathBuf, BackupError> {
    let destination = directory.join(name);
    std::fs::copy(source, &destination).map_err(|source| BackupError::LocalFile {
        operation: "keep".to_owned(),
        path: destination.to_string_lossy().into_owned(),
        source,
    })?;
    Ok(destination)
}

#[cfg(test)]
#[path = "keep_archive_test.rs"]
mod keep_archive_test;
