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
    ///
    /// Trusted input only. A key that came from a person goes through
    /// [`ArchiveName::parse_for`] instead, which checks it before it reaches a
    /// path join.
    pub fn from_key(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Adopts a key a person typed, refusing anything this tool did not write
    /// for `label`.
    ///
    /// A key becomes a local filename when the archive is downloaded, so an
    /// unchecked one carrying `../` escapes the scratch directory and writes
    /// wherever it points. Matching the label is not enough on its own:
    /// `app-backup-../../x.dump` clears that test. The stamp has to be a stamp,
    /// which is what rules out a separator, and it also catches the common slip
    /// of passing the `.sha256` sidecar instead of the dump.
    pub fn parse_for(key: &str, label: &str) -> Option<Self> {
        let stamp = Self::belongs_to(key, label).then(|| stamp_of(key))??;

        is_stamp(stamp).then(|| Self(key.to_owned()))
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

    /// The UTC stamp embedded in the key, e.g. `20260803-0942`.
    ///
    /// Read back out of the name rather than from object metadata, so asking
    /// "when did the last backup land" costs one bucket listing instead of a
    /// request per candidate. `None` for a key that is not one of ours, which is
    /// the same question `belongs_to` answers and the reason this cannot guess.
    pub fn stamp(&self) -> Option<&str> {
        stamp_of(&self.0)
    }
}

/// The stamp text in `key`, without checking that it reads as a time.
///
/// From the right: a label may itself contain `-backup-`, and the stamp is
/// always the last thing before the extension.
fn stamp_of(key: &str) -> Option<&str> {
    let separator = format!("-{ARCHIVE_STEM}-");
    let extension = format!(".{}", ArchiveFile::Dump.extension());

    let (_, after_stem) = key.rsplit_once(&separator)?;
    after_stem.strip_suffix(&extension)
}

/// True when `text` has the shape `ArchiveName::new` writes: `YYYYmmdd-HHMM`.
///
/// Shape only, not a calendar check. Reading it as a time is the scheduler's
/// job; here it exists so a key that reached a path join cannot carry a
/// directory separator or a `..` segment.
fn is_stamp(text: &str) -> bool {
    let Some((date, time)) = text.split_once('-') else {
        return false;
    };

    date.len() == 8
        && time.len() == 4
        && date
            .bytes()
            .chain(time.bytes())
            .all(|byte| byte.is_ascii_digit())
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
