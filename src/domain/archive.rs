//! Archive identity: what a backup object is called and how its integrity is
//! stated. Pure values -- no filesystem, no network.

use std::fmt;

/// Filename stem shared by every archive this tool writes.
const ARCHIVE_STEM: &str = "backup";

/// The two files one backup produces. A backup is always this pair, so the
/// extensions are variants of one choice rather than unrelated constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFile {
    /// The `pg_dump` custom-format archive.
    Dump,
    /// The sidecar holding the archive's hex SHA-256.
    Checksum,
}

impl ArchiveFile {
    /// The filename extension this file kind carries, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Dump => "dump",
            Self::Checksum => "sha256",
        }
    }
}

/// The object key of one backup, e.g. `app-backup-20260803-0942.dump`.
///
/// Built from a database label plus a UTC timestamp so keys sort
/// chronologically as plain strings, which is how `verify` picks the newest
/// archive without reading object metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArchiveName(String);

impl ArchiveName {
    /// Builds the key for `label` stamped with `utc_stamp` (`YYYYmmdd-HHMM`).
    pub fn new(label: &str, utc_stamp: &str) -> Self {
        Self(format!(
            "{label}-{ARCHIVE_STEM}-{utc_stamp}.{}",
            ArchiveFile::Dump.extension()
        ))
    }

    /// Adopts an existing object key, e.g. one listed from the bucket.
    pub fn from_key(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The key of the checksum sidecar that travels with this archive.
    pub fn checksum_key(&self) -> String {
        format!("{}.{}", self.0, ArchiveFile::Checksum.extension())
    }

    /// True when `key` is an archive this tool produced for `label`.
    ///
    /// The label is part of the test, not decoration. A bucket may hold keys
    /// from an older naming scheme or from a neighbouring project, and sorting
    /// across two prefixes is not chronological: `app-backup-20260803` sorts
    /// BEFORE `legacy-backup-20260708`, so a mixed listing would hand back a
    /// stale archive as the newest one. Anchoring on `<label>-backup-` keeps
    /// every candidate in one scheme, where string order is time order.
    pub fn belongs_to(key: &str, label: &str) -> bool {
        key.starts_with(&format!("{label}-{ARCHIVE_STEM}-"))
            && key.ends_with(&format!(".{}", ArchiveFile::Dump.extension()))
    }

    /// The key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArchiveName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A hex-encoded SHA-256 digest of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveDigest(String);

impl ArchiveDigest {
    /// Wraps an already-computed lowercase hex digest.
    pub fn from_hex(digest: impl Into<String>) -> Self {
        Self(digest.into())
    }

    /// Parses the sidecar body written by this tool, which follows the
    /// `sha256sum` layout: `<hex>  <filename>`.
    pub fn from_sidecar(body: &str) -> Option<Self> {
        body.split_whitespace().next().map(Self::from_hex)
    }

    /// Renders the sidecar body for `archive`, byte-compatible with
    /// `sha256sum -c` so the archive stays verifiable without this tool.
    pub fn to_sidecar(&self, archive: &ArchiveName) -> String {
        format!("{}  {}\n", self.0, archive)
    }

    /// The digest as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArchiveDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "archive_test.rs"]
mod archive_test;
