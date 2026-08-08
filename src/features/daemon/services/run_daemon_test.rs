use super::PassOutcome;
use crate::domain::Interval;

#[test]
fn a_deferred_pass_carries_the_time_left_rather_than_a_bare_flag() {
    // The loop sleeps by exactly this, so a container that restarts an hour
    // into a daily cadence waits the remaining 23 hours instead of a full day.
    let outcome = PassOutcome::Deferred {
        remaining: Interval::from_secs(23 * 60 * 60),
    };

    let PassOutcome::Deferred { remaining } = outcome else {
        panic!("expected a deferred pass");
    };
    assert_eq!(remaining.to_string(), "23h");
}

#[test]
fn a_completed_pass_reports_what_it_stored_and_what_it_removed() {
    let outcome = PassOutcome::BackedUp {
        stored: "app-backup-20260803-0942.dump".to_owned(),
        deleted: 2,
    };

    let PassOutcome::BackedUp { stored, deleted } = outcome else {
        panic!("expected a completed pass");
    };
    assert_eq!(stored, "app-backup-20260803-0942.dump");
    assert_eq!(deleted, 2);
}
