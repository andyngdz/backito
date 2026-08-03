//! What a completed backup consists of.

use std::path::PathBuf;

use crate::domain::{ArchiveDigest, ArchiveName};

/// The result of one backup run.
#[derive(Debug, Clone)]
pub struct BackupOutcome {
    /// Object key the archive was stored under.
    pub archive: ArchiveName,
    /// SHA-256 of the archive.
    pub digest: ArchiveDigest,
    /// Size of the archive on disk.
    pub local_bytes: u64,
    /// Size the store reports for the uploaded object. Compared against
    /// `local_bytes` by the caller: a mismatch means a truncated upload.
    pub stored_bytes: u64,
    /// Tables the archive carries data for.
    pub tables: usize,
    /// Where the archive was written locally.
    pub local_path: PathBuf,
}

impl BackupOutcome {
    /// True when the stored object is byte-for-byte the size of what was sent.
    pub fn sizes_match(&self) -> bool {
        self.local_bytes == self.stored_bytes
    }
}

#[cfg(test)]
#[path = "domain_test.rs"]
mod domain_test;
