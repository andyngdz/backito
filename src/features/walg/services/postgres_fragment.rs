//! The Postgres configuration that turns WAL archiving on.

/// How long Postgres may sit on a partially filled segment before archiving it.
///
/// Without this a quiet database archives nothing for hours, and the recovery
/// window is however long since the last segment filled rather than ten minutes.
const ARCHIVE_TIMEOUT_SECONDS: u32 = 600;

/// The settings that turn archiving on, pointed at this backito and this source.
pub fn archiving_fragment(source_flags: &str) -> String {
    format!(
        "archive_mode = on\n\
         archive_command = '{}'\n\
         archive_timeout = {ARCHIVE_TIMEOUT_SECONDS}\n",
        archive_invocation(source_flags)
    )
}

/// What to write when there is no WAL storage.
pub fn disabled_fragment() -> String {
    "# WAL archiving is off: the backito config carries no [walg] section\n".to_owned()
}

/// The command Postgres runs for each segment.
///
/// The executable path is taken from this process rather than assumed, and the
/// source flags are repeated verbatim. Postgres runs `archive_command` from its
/// own working directory with a minimal environment, so a bare `backito` may not
/// be on PATH and a relative `backito.toml` would not be found.
fn archive_invocation(source_flags: &str) -> String {
    let executable = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "backito".to_owned());

    format!("{executable} {source_flags} walg archive %p")
}

#[cfg(test)]
#[path = "postgres_fragment_test.rs"]
mod postgres_fragment_test;
