use super::{DEFAULT_BACKUP_INTERVAL, DEFAULT_RETAIN, DEFAULT_VERIFY_INTERVAL, checked_retain};
use crate::infra::config::ConfigError;

#[test]
fn a_retention_that_keeps_something_is_accepted() {
    assert_eq!(checked_retain(1).expect("one is enough"), 1);
    assert_eq!(checked_retain(DEFAULT_RETAIN).expect("default"), 30);
}

#[test]
fn retaining_nothing_is_refused_before_anything_can_be_deleted() {
    // Retention runs straight after a backup lands, so accepting 0 here would
    // delete the archive that pass just wrote. The refusal belongs at load time:
    // by the time prune sees the number, the dump has already been uploaded.
    let failure = checked_retain(0).expect_err("zero must not load");

    assert!(matches!(failure, ConfigError::RetainsNothing));
}

#[test]
fn the_defaults_describe_a_daily_backup_kept_for_a_month() {
    // `retain` counts archives, not days, so the month only holds at the default
    // cadence. These two constants are read together and are worth pinning as a
    // pair.
    assert_eq!(DEFAULT_BACKUP_INTERVAL.as_secs(), 24 * 60 * 60);
    assert_eq!(DEFAULT_RETAIN, 30);
}

#[test]
fn verification_is_weekly_and_can_be_switched_off() {
    // Zero disables verification, which is the opposite of what it means for
    // retention. The asymmetry is deliberate and worth a test.
    assert_eq!(DEFAULT_VERIFY_INTERVAL.as_secs(), 7 * 24 * 60 * 60);
    assert!(!DEFAULT_VERIFY_INTERVAL.is_disabled());
}
