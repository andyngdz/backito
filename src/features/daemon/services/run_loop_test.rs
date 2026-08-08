use super::verify_is_due;
use crate::domain::Interval;

fn interval(text: &str) -> Interval {
    text.parse().expect("parse")
}

#[test]
fn a_zero_cadence_disables_verification_however_long_it_has_been() {
    assert!(!verify_is_due(interval("0s"), interval("30d")));
}

#[test]
fn verification_waits_until_its_cadence_has_passed() {
    assert!(!verify_is_due(interval("7d"), interval("6d")));
}

#[test]
fn verification_is_due_once_the_cadence_is_reached() {
    assert!(verify_is_due(interval("7d"), interval("7d")));
    assert!(verify_is_due(interval("7d"), interval("8d")));
}
