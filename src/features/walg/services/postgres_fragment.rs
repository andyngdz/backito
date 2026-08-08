//! The Postgres configuration that turns WAL archiving on.

use std::path::Path;

/// How long Postgres may sit on a partially filled segment before archiving it.
///
/// Without this a quiet database archives nothing for hours, and the recovery
/// window is however long since the last segment filled rather than ten minutes.
const ARCHIVE_TIMEOUT_SECONDS: u32 = 600;

/// The settings that turn archiving on, pointed at this backito and this config.
pub fn archiving_fragment(config_path: &Path) -> String {
    format!(
        "archive_mode = on\n\
         archive_command = '{}'\n\
         archive_timeout = {ARCHIVE_TIMEOUT_SECONDS}\n",
        archive_invocation(config_path)
    )
}

/// What to write when there is no WAL storage.
pub fn disabled_fragment() -> String {
    "# WAL archiving is off: the backito config carries no [walg] section\n".to_owned()
}

/// The command Postgres runs for each segment.
///
/// Both paths are absolute and taken from this process rather than assumed.
/// Postgres runs `archive_command` from its own working directory with a
/// minimal environment, so a bare `backito` may not be on PATH and a relative
/// `backito.toml` would not be found.
fn archive_invocation(config_path: &Path) -> String {
    let executable = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "backito".to_owned());

    format!(
        "{executable} --config {} walg archive %p",
        config_path.display()
    )
}

#[cfg(test)]
#[path = "postgres_fragment_test.rs"]
mod postgres_fragment_test;
