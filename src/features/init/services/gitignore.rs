//! Keeps the config file out of git.
//!
//! `backito.toml` names a bucket and an S3 endpoint, and an R2 endpoint carries
//! the account id. That is per-machine setup rather than shared source, so the
//! entry is added for the user instead of being left as a README instruction
//! nobody follows.

use std::path::{Path, PathBuf};

use super::super::{FileOperation, IgnoreOutcome, InitError};

/// Name of the ignore file this touches.
const IGNORE_FILENAME: &str = ".gitignore";

/// Comment written above the entry, so a reader knows why it is there.
const ENTRY_COMMENT: &str = "# backito config: names a bucket and an S3 endpoint, kept out of git";

/// Ensures `entry` is ignored by the `.gitignore` in `directory`.
pub fn ensure_ignored(directory: &Path, entry: &str) -> Result<IgnoreOutcome, InitError> {
    let path = directory.join(IGNORE_FILENAME);

    let existing = match std::fs::read_to_string(&path) {
        Ok(body) => Some(body),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => None,
        Err(source) => return Err(read_failure(&path, source)),
    };

    let Some(body) = existing else {
        write_ignore(&path, &format!("{ENTRY_COMMENT}\n{entry}\n"))?;
        return Ok(IgnoreOutcome::Created { path });
    };

    if covers(&body, entry) {
        return Ok(IgnoreOutcome::AlreadyIgnored { path });
    }

    let separator = if body.ends_with('\n') || body.is_empty() {
        ""
    } else {
        "\n"
    };
    write_ignore(
        &path,
        &format!("{body}{separator}\n{ENTRY_COMMENT}\n{entry}\n"),
    )?;
    Ok(IgnoreOutcome::Appended { path })
}

/// True when `body` already ignores `entry`.
///
/// Compares whole lines: a substring match would treat `backito.toml.bak` as
/// covering `backito.toml` and skip an entry that is actually missing.
pub fn covers(body: &str, entry: &str) -> bool {
    body.lines()
        .map(str::trim)
        .any(|line| line == entry || line == format!("/{entry}"))
}

/// Writes the ignore file.
fn write_ignore(path: &Path, body: &str) -> Result<(), InitError> {
    std::fs::write(path, body).map_err(|source| InitError::File {
        operation: FileOperation::Write,
        path: path.to_path_buf(),
        source,
    })
}

/// Builds the failure for an unreadable ignore file.
fn read_failure(path: &Path, source: std::io::Error) -> InitError {
    InitError::File {
        operation: FileOperation::Read,
        path: PathBuf::from(path),
        source,
    }
}

#[cfg(test)]
#[path = "gitignore_test.rs"]
mod gitignore_test;
