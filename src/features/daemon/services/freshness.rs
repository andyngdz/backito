//! Is the newest backup recent enough to be worth having?
//!
//! This is what a container healthcheck asks. A liveness probe cannot answer it:
//! a backup loop that is running fine but has stopped being able to upload looks
//! exactly like one that works, and that is the failure worth catching.

use jiff::Timestamp;

use super::due::{ArchiveAge, NewestArchive, archive_age};
use crate::domain::Interval;

/// How many backup intervals may pass before the newest archive is stale.
///
/// One missed backup is a retry; two is a pattern. At the default daily cadence
/// this trips within a day of the first failure, which is soon enough to act on
/// and late enough not to page anyone over a single slow night.
const STALE_AFTER_INTERVALS: u32 = 2;

/// The verdict a healthcheck reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupFreshness {
    /// A backup landed within the budget.
    Fresh {
        /// How long ago it landed.
        age: Interval,
    },
    /// A backup exists but is older than the budget allows.
    Stale {
        /// How long ago it landed.
        age: Interval,
        /// The budget it exceeded.
        budget: Interval,
    },
    /// Nothing readable to measure: an empty bucket, a key with no stamp, or a
    /// stamp ahead of the clock. All three mean the same thing to an operator,
    /// which is that no backup can be shown to exist.
    Unknown,
}

/// The staleness budget for a given backup cadence.
pub(super) fn staleness_budget(cadence: Interval) -> Interval {
    cadence.times(STALE_AFTER_INTERVALS)
}

/// Judges the newest archive against the budget derived from `cadence`.
pub fn backup_freshness(
    newest: &NewestArchive,
    cadence: Interval,
    now: Timestamp,
) -> BackupFreshness {
    let budget = staleness_budget(cadence);

    match archive_age(newest, now) {
        ArchiveAge::Unknown => BackupFreshness::Unknown,
        ArchiveAge::Known(age) if age <= budget => BackupFreshness::Fresh { age },
        ArchiveAge::Known(age) => BackupFreshness::Stale { age, budget },
    }
}

#[cfg(test)]
#[path = "freshness_test.rs"]
mod freshness_test;
