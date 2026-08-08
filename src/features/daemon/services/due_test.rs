use super::{ArchiveAge, BackupDue, NewestArchive, archive_age, backup_due};
use crate::domain::Interval;
use jiff::Timestamp;

/// `20260803-1200` as an instant, so the tests can sit either side of it.
fn taken_at_noon() -> Timestamp {
    "2026-08-03T12:00:00Z".parse().expect("a fixed instant")
}

fn stamped(stamp: &str) -> NewestArchive {
    NewestArchive::Stamped(stamp.to_owned())
}

fn one_day() -> Interval {
    "24h".parse().expect("parse")
}

#[test]
fn nothing_stored_means_back_up_now() {
    let verdict = backup_due(&NewestArchive::Absent, one_day(), taken_at_noon());

    assert_eq!(verdict, BackupDue::Now);
}

#[test]
fn an_unreadable_stamp_means_back_up_now() {
    let verdict = backup_due(
        &NewestArchive::Stamped("not-a-stamp".to_owned()),
        one_day(),
        taken_at_noon(),
    );

    assert_eq!(verdict, BackupDue::Now);
}

#[test]
fn an_archive_older_than_the_interval_is_due() {
    let now: Timestamp = "2026-08-04T12:00:01Z".parse().expect("parse");

    let verdict = backup_due(&stamped("20260803-1200"), one_day(), now);

    assert_eq!(verdict, BackupDue::Now);
}

#[test]
fn an_archive_exactly_one_interval_old_is_due() {
    let now: Timestamp = "2026-08-04T12:00:00Z".parse().expect("parse");

    let verdict = backup_due(&stamped("20260803-1200"), one_day(), now);

    assert_eq!(verdict, BackupDue::Now);
}

#[test]
fn a_fresh_archive_defers_by_exactly_what_is_left() {
    // One hour after the last backup, 23 hours of the day remain.
    let now: Timestamp = "2026-08-03T13:00:00Z".parse().expect("parse");

    let verdict = backup_due(&stamped("20260803-1200"), one_day(), now);

    assert_eq!(
        verdict,
        BackupDue::NotUntil {
            remaining: Interval::from_secs(23 * 60 * 60)
        }
    );
}

#[test]
fn a_restart_seconds_after_a_backup_does_not_dump_again() {
    // This is the behaviour the shell loop lacked: it began every start with a
    // full dump, so redeploying three times produced three archives.
    let now: Timestamp = "2026-08-03T12:00:30Z".parse().expect("parse");

    let verdict = backup_due(&stamped("20260803-1200"), one_day(), now);

    assert!(matches!(verdict, BackupDue::NotUntil { .. }));
}

#[test]
fn a_stamp_in_the_future_means_back_up_rather_than_wait() {
    // A stamp ahead of the clock means something is wrong, and the two ways to
    // be wrong are not symmetric: an extra archive costs storage, while trusting
    // a clock that is a year fast costs a year of backups.
    let now: Timestamp = "2026-08-03T11:00:00Z".parse().expect("parse");

    let verdict = backup_due(&stamped("20260803-1200"), one_day(), now);

    assert_eq!(verdict, BackupDue::Now);
}

#[test]
fn the_age_of_a_readable_stamp_is_the_time_since_it_was_taken() {
    let now: Timestamp = "2026-08-03T15:30:00Z".parse().expect("parse");

    let age = archive_age(&stamped("20260803-1200"), now);

    assert_eq!(
        age,
        ArchiveAge::Known(Interval::from_secs(3 * 60 * 60 + 30 * 60))
    );
}

#[test]
fn an_age_cannot_be_measured_without_a_readable_stamp() {
    for newest in [
        NewestArchive::Absent,
        NewestArchive::Unstamped,
        NewestArchive::Stamped("not-a-stamp".to_owned()),
    ] {
        assert_eq!(archive_age(&newest, taken_at_noon()), ArchiveAge::Unknown);
    }
}

#[test]
fn an_archive_whose_key_carries_no_stamp_means_back_up_now() {
    let verdict = backup_due(&NewestArchive::Unstamped, one_day(), taken_at_noon());

    assert_eq!(verdict, BackupDue::Now);
}
