//! Whether a backup is due, decided from the newest archive already stored.
//!
//! Pure: the clock and the bucket's answer are both parameters. The bucket is
//! the source of truth on purpose. A local marker file would reset with the
//! container, and resetting is exactly what makes a restarted service dump the
//! database again.

use jiff::Timestamp;
use jiff::civil::DateTime;

use crate::domain::Interval;

/// Format the archive stamp is written in, e.g. `20260803-0942`.
const STAMP_FORMAT: &str = "%Y%m%d-%H%M";

/// What the bucket says about the most recent archive for a label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewestArchive {
    /// An archive exists and its key carries a readable stamp.
    Stamped(String),
    /// The bucket holds no archive for this label. A normal first run.
    Absent,
    /// An archive exists, but its key carries no stamp this tool can read.
    /// Kept apart from `Absent` because it means something else wrote a key
    /// that passes the label filter, which is worth saying out loud.
    Unstamped,
}

/// What the schedule says to do right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupDue {
    /// Nothing recent enough exists; back up now.
    Now,
    /// A recent archive covers this interval. Carries how long is left, so the
    /// caller sleeps exactly that long rather than a whole interval.
    NotUntil {
        /// Time left before the next backup is due.
        remaining: Interval,
    },
}

/// Decides whether to back up, given what the bucket holds.
///
/// Anything other than a readable, recent stamp means back up. Any doubt about
/// when the last backup happened resolves towards taking one: a spare archive
/// costs storage, a missing one costs the database.
pub fn backup_due(newest: &NewestArchive, interval: Interval, now: Timestamp) -> BackupDue {
    due_from_age(archive_age(newest, now), interval)
}

/// The same decision, from an age measured some other way.
///
/// Physical backups are listed by `wal-g` rather than by key, so their age comes
/// from a different place. The rule about what to do with it is the same one,
/// and is worth having in a single place with a single set of tests.
pub fn due_from_age(age: ArchiveAge, interval: Interval) -> BackupDue {
    let ArchiveAge::Known(age) = age else {
        return BackupDue::Now;
    };

    match interval.as_secs().checked_sub(age.as_secs()) {
        Some(0) | None => BackupDue::Now,
        Some(remaining) => BackupDue::NotUntil {
            remaining: Interval::from_secs(remaining),
        },
    }
}

/// How old the newest archive is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveAge {
    /// The archive exists and its stamp reads as an instant in the past.
    Known(Interval),
    /// There is nothing to measure: no archive, no readable stamp, or a stamp
    /// ahead of the clock. Every caller treats all three the same way, so they
    /// share one variant rather than inviting three identical match arms.
    Unknown,
}

/// How long ago the newest archive was taken.
///
/// A stamp in the future reads as `Unknown` rather than as a negative age: it
/// means a clock moved, and no caller has a sensible answer for "taken in three
/// hours' time".
pub fn archive_age(newest: &NewestArchive, now: Timestamp) -> ArchiveAge {
    let NewestArchive::Stamped(stamp) = newest else {
        return ArchiveAge::Unknown;
    };

    let Some(taken_at) = parse_stamp(stamp) else {
        return ArchiveAge::Unknown;
    };

    let elapsed = now.as_second() - taken_at.as_second();
    if elapsed < 0 {
        return ArchiveAge::Unknown;
    }

    ArchiveAge::Known(Interval::from_secs(elapsed.unsigned_abs()))
}

/// Reads a stamp as a UTC instant, or `None` when it is not one.
fn parse_stamp(stamp: &str) -> Option<Timestamp> {
    DateTime::strptime(STAMP_FORMAT, stamp)
        .ok()?
        .to_zoned(jiff::tz::TimeZone::UTC)
        .ok()
        .map(|zoned| zoned.timestamp())
}

#[cfg(test)]
#[path = "due_test.rs"]
mod due_test;
