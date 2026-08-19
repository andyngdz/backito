//! Archive identity: what a backup object is called and how its integrity is
//! stated. Pure values -- no filesystem, no network.

use std::fmt;

use super::ArchiveKeyError;

use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;

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

        stamp_taken_at(stamp).map(|_| Self(key.to_owned()))
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

/// Format the UTC stamp inside a key is written and read in.
///
/// One definition, because the writer and the reader have to agree. When they
/// drifted apart the daemon read every archive as undatable, which reads as
/// "nothing has been backed up" and dumps the database on every pass.
const STAMP_FORMAT: &str = "%Y%m%d-%H%M";

/// Renders `moment` as an archive key carries it.
///
/// Takes the instant rather than reading the clock, so the caller owns the
/// time and this stays pure.
pub fn stamp_at(moment: Timestamp) -> String {
    moment.strftime(STAMP_FORMAT).to_string()
}

/// Reads a stamp back as the UTC instant it names, or `None` when it is not one.
///
/// Accepts only the canonical spelling, checked by rendering the parsed instant
/// again and requiring the same text. `strptime` alone is looser than the format
/// suggests: it reads `%H%M` out of `094` as 09:04, so a truncated key would
/// pass. Insisting on a round trip needs no second set of rules to stay in step
/// with `stamp_at`.
///
/// This is also what makes a typed key safe to join onto a path. A canonical
/// stamp is digits and one hyphen, so nothing that parses here carries a
/// directory separator or a `..` segment.
pub fn stamp_taken_at(stamp: &str) -> Option<Timestamp> {
    let moment = DateTime::strptime(STAMP_FORMAT, stamp)
        .ok()?
        .to_zoned(TimeZone::UTC)
        .ok()
        .map(|zoned| zoned.timestamp())?;

    (stamp_at(moment) == stamp).then_some(moment)
}

impl fmt::Display for ArchiveName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Which archive a command was told to work on.
///
/// An enum rather than `Option<ArchiveName>` because the absent case is a real
/// instruction, not a missing value: it means "whichever is newest", which is
/// resolved against the bucket rather than defaulted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveChoice {
    /// Whichever archive for this label is newest.
    Newest,
    /// This archive, named by a person and already checked.
    Named(ArchiveName),
}

impl ArchiveChoice {
    /// Reads a `--archive` value, refusing a key this tool did not write.
    ///
    /// Lives here so `verify` and `restore` get the same check without either
    /// owning it, and so the check sits next to the naming rules it enforces.
    pub fn parse(key: Option<String>, label: &str) -> Result<Self, ArchiveKeyError> {
        let Some(key) = key else {
            return Ok(Self::Newest);
        };

        ArchiveName::parse_for(&key, label)
            .map(Self::Named)
            .ok_or(ArchiveKeyError::NotOurs {
                key,
                label: label.to_owned(),
            })
    }
}

/// One archive as a bucket holds it: what it is called, and how big it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredArchive {
    /// Key the archive is stored under.
    pub name: ArchiveName,
    /// Size the store reports for it.
    pub bytes: u64,
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
