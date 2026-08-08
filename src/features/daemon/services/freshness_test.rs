use super::super::due::NewestArchive;
use super::{BackupFreshness, backup_freshness, staleness_budget};
use crate::domain::Interval;
use jiff::Timestamp;

fn one_day() -> Interval {
    "24h".parse().expect("parse")
}

fn stamped(stamp: &str) -> NewestArchive {
    NewestArchive::Stamped(stamp.to_owned())
}

fn at(instant: &str) -> Timestamp {
    instant.parse().expect("a fixed instant")
}

#[test]
fn the_budget_is_two_cadences() {
    assert_eq!(staleness_budget(one_day()).as_secs(), 48 * 60 * 60);
}

#[test]
fn a_backup_from_this_morning_is_fresh() {
    let verdict = backup_freshness(
        &stamped("20260803-1200"),
        one_day(),
        at("2026-08-03T18:00:00Z"),
    );

    assert_eq!(
        verdict,
        BackupFreshness::Fresh {
            age: Interval::from_secs(6 * 60 * 60)
        }
    );
    assert!(matches!(verdict, BackupFreshness::Fresh { .. }));
}

#[test]
fn one_missed_backup_is_still_healthy() {
    // 30 hours: past the daily cadence, inside the two-day budget. A single
    // late night is a retry, not an incident.
    let verdict = backup_freshness(
        &stamped("20260803-1200"),
        one_day(),
        at("2026-08-04T18:00:00Z"),
    );

    assert!(matches!(verdict, BackupFreshness::Fresh { .. }));
}

#[test]
fn two_missed_backups_are_stale() {
    let verdict = backup_freshness(
        &stamped("20260803-1200"),
        one_day(),
        at("2026-08-05T13:00:00Z"),
    );

    assert_eq!(
        verdict,
        BackupFreshness::Stale {
            age: Interval::from_secs(49 * 60 * 60),
            budget: Interval::from_secs(48 * 60 * 60),
        }
    );
    assert!(!matches!(verdict, BackupFreshness::Fresh { .. }));
}

#[test]
fn an_archive_exactly_at_the_budget_is_still_fresh() {
    let verdict = backup_freshness(
        &stamped("20260803-1200"),
        one_day(),
        at("2026-08-05T12:00:00Z"),
    );

    assert!(matches!(verdict, BackupFreshness::Fresh { .. }));
}

#[test]
fn an_empty_bucket_is_not_healthy() {
    let verdict = backup_freshness(
        &NewestArchive::Absent,
        one_day(),
        at("2026-08-03T12:00:00Z"),
    );

    assert_eq!(verdict, BackupFreshness::Unknown);
    assert!(!matches!(verdict, BackupFreshness::Fresh { .. }));
}
