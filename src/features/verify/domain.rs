//! What a verification produced.

use crate::domain::{ArchiveName, TableComparison};

/// The result of restoring an archive into a scratch database and comparing it
/// to the live source.
#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    /// Archive that was restored.
    pub archive: ArchiveName,
    /// Per-table comparison against the live source.
    pub comparisons: Vec<TableComparison>,
    /// Rows the restored copy is behind across drifting tables.
    pub rows_behind: i64,
    /// Errors `pg_restore` reported. Kept for display only: a managed Postgres
    /// image reports dozens of them for system objects it already owns, none of
    /// which mean application rows failed to land.
    pub restore_errors: usize,
    /// Whether the stored checksum matched the bytes that came back.
    pub checksum: ChecksumOutcome,
}

/// How the downloaded archive compared to its stored checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChecksumOutcome {
    /// The sidecar matched the downloaded bytes.
    Matched,
    /// The sidecar disagreed with the downloaded bytes.
    Mismatched {
        /// Digest recorded when the archive was uploaded.
        expected: String,
        /// Digest of what came back.
        actual: String,
    },
    /// No sidecar was stored beside the archive.
    Absent,
}

impl VerifyOutcome {
    /// Tables whose counts cannot be explained by drift.
    pub fn failures(&self) -> Vec<&TableComparison> {
        self.comparisons
            .iter()
            .filter(|comparison| comparison.verdict.is_failure())
            .collect()
    }

    /// True when the archive restored to a database that matches its source.
    ///
    /// Row counts and the checksum decide this. `pg_restore`'s exit code and
    /// error tally deliberately do not. A missing sidecar fails: an archive
    /// whose bytes cannot be checked has not been verified, only restored.
    pub fn passed(&self) -> bool {
        self.failures().is_empty() && self.checksum == ChecksumOutcome::Matched
    }
}

#[cfg(test)]
#[path = "domain_test.rs"]
mod domain_test;
