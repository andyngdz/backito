use super::BackupOutcome;
use crate::domain::{ArchiveDigest, ArchiveName};
use std::path::PathBuf;

fn outcome(local: u64, stored: u64) -> BackupOutcome {
    BackupOutcome {
        archive: ArchiveName::new("app", "20260803-0942"),
        digest: ArchiveDigest::from_hex("abc"),
        local_bytes: local,
        stored_bytes: stored,
        tables: 44,
        local_path: PathBuf::from("/tmp/archive.dump"),
    }
}

#[test]
fn equal_sizes_match() {
    assert!(outcome(925_177_935, 925_177_935).sizes_match());
}

#[test]
fn a_short_upload_does_not_match() {
    // A stored object smaller than the source is a truncated upload, which the
    // store itself reports as success.
    assert!(!outcome(925_177_935, 925_000_000).sizes_match());
}
