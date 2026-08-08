//! Reading `wal-g backup-list` well enough to know when the last base backup
//! landed.
//!
//! wal-g prints a fixed-width table with a header row:
//!
//! ```text
//! backup_name                   modified             wal_file_name            storage_name
//! base_000000010000000400000025 2026-08-08T18:16:20Z 000000010000000400000025 default
//! ```
//!
//! Only the `modified` column is read here. Everything else on the line is
//! wal-g's business, and parsing less means a future column cannot break this.

use jiff::Timestamp;

use crate::domain::Interval;
use crate::features::daemon::ArchiveAge;

/// Prefix wal-g gives every base backup it writes.
const BASE_PREFIX: &str = "base_";

/// How old the newest base backup is, from the output of `backup-list`.
///
/// Unreadable output reads as `Unknown`, which every caller turns into "take a
/// backup". wal-g changing its table layout should cost an extra base backup,
/// not a silent gap where none is taken at all.
pub fn newest_base_age(listing: &str, now: Timestamp) -> ArchiveAge {
    let newest = listing
        .lines()
        .filter_map(parse_modified)
        .max_by_key(|taken_at| taken_at.as_second());

    let Some(taken_at) = newest else {
        return ArchiveAge::Unknown;
    };

    let elapsed = now.as_second() - taken_at.as_second();
    if elapsed < 0 {
        return ArchiveAge::Unknown;
    }

    ArchiveAge::Known(Interval::from_secs(elapsed.unsigned_abs()))
}

/// The `modified` timestamp on one listing row, or `None` for a row that is not
/// a base backup.
///
/// The header row and any progress lines wal-g writes fall out here rather than
/// being counted, because neither starts with a `base_` name.
fn parse_modified(line: &str) -> Option<Timestamp> {
    let mut columns = line.split_whitespace();
    let name = columns.next()?;
    if !name.starts_with(BASE_PREFIX) {
        return None;
    }

    columns.next()?.parse().ok()
}

#[cfg(test)]
#[path = "backup_list_test.rs"]
mod backup_list_test;
