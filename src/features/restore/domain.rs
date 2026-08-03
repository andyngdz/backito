//! What a restore produced.

use crate::domain::ArchiveName;

/// The result of loading an archive into a real database.
#[derive(Debug, Clone)]
pub struct RestoreOutcome {
    /// Archive that was restored.
    pub archive: ArchiveName,
    /// Size of the archive that was fetched.
    pub bytes: u64,
    /// Everything `pg_restore` wrote to stderr, kept for display.
    pub restore_stderr: String,
}

#[cfg(test)]
#[path = "domain_test.rs"]
mod domain_test;
