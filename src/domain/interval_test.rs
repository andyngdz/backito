use super::Interval;
use crate::domain::IntervalError;

#[test]
fn every_unit_reads_as_the_seconds_it_stands_for() {
    let cases = [
        ("30s", 30),
        ("15m", 15 * 60),
        ("24h", 24 * 60 * 60),
        ("7d", 7 * 24 * 60 * 60),
    ];

    for (text, seconds) in cases {
        let parsed: Interval = text.parse().expect("a well-formed interval parses");
        assert_eq!(parsed.as_secs(), seconds, "reading {text}");
    }
}

#[test]
fn a_day_is_twenty_four_hours_here() {
    let day: Interval = "1d".parse().expect("parse");
    let hours: Interval = "24h".parse().expect("parse");

    assert_eq!(day, hours);
}

#[test]
fn surrounding_whitespace_is_ignored() {
    let parsed: Interval = "  12h  ".parse().expect("parse");

    assert_eq!(parsed.as_secs(), 12 * 60 * 60);
}

#[test]
fn a_bare_number_is_refused_rather_than_assumed_to_be_seconds() {
    let failure = "3600".parse::<Interval>().expect_err("no unit given");

    // The last character decides the unit, so a bare number fails as a missing
    // unit rather than a bad number. Either way it never silently means seconds.
    assert_eq!(
        failure,
        IntervalError::UnknownUnit {
            text: "3600".to_owned()
        }
    );
}

#[test]
fn a_unit_without_a_number_is_refused() {
    let failure = "h".parse::<Interval>().expect_err("no amount given");

    assert_eq!(
        failure,
        IntervalError::NotANumber {
            text: "h".to_owned()
        }
    );
}

#[test]
fn an_unknown_unit_names_what_is_accepted() {
    let failure = "5w".parse::<Interval>().expect_err("weeks are not a unit");

    assert!(failure.to_string().contains("s, m, h, or d"));
}

#[test]
fn an_empty_interval_is_refused() {
    let failure = "   ".parse::<Interval>().expect_err("nothing to read");

    assert_eq!(failure, IntervalError::Empty);
}

#[test]
fn zero_reads_as_disabled_rather_than_as_run_continuously() {
    let parsed: Interval = "0s".parse().expect("parse");

    assert!(parsed.is_disabled());
}

#[test]
fn times_scales_a_cadence_into_a_staleness_budget() {
    let cadence: Interval = "24h".parse().expect("parse");

    assert_eq!(cadence.times(2).as_secs(), 48 * 60 * 60);
}

#[test]
fn display_canonicalises_to_the_largest_whole_unit() {
    let cases = [
        ("30s", "30s"),
        ("15m", "15m"),
        ("90s", "90s"),
        ("120s", "2m"),
        ("24h", "1d"),
        ("36h", "36h"),
        ("7d", "7d"),
    ];

    for (written, canonical) in cases {
        let parsed: Interval = written.parse().expect("parse");

        assert_eq!(parsed.to_string(), canonical, "canonicalising {written}");
    }
}
