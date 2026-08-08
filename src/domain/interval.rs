//! How long to wait between scheduled runs. A pure value: no clock, no config.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use super::IntervalError;

/// The units an interval may be written in.
///
/// A closed set, so it is an enum rather than a string compared in four places.
/// The only string handling is `from_suffix`, at the parse boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntervalUnit {
    Second,
    Minute,
    Hour,
    Day,
}

impl IntervalUnit {
    /// Reads the trailing character of an interval, e.g. the `h` of `24h`.
    fn from_suffix(suffix: char) -> Option<Self> {
        match suffix {
            's' => Some(Self::Second),
            'm' => Some(Self::Minute),
            'h' => Some(Self::Hour),
            'd' => Some(Self::Day),
            _ => None,
        }
    }

    /// Seconds in one of this unit.
    const fn seconds(self) -> u64 {
        match self {
            Self::Second => 1,
            Self::Minute => 60,
            Self::Hour => 60 * 60,
            Self::Day => 24 * 60 * 60,
        }
    }

    /// The character this unit is written with.
    const fn suffix(self) -> char {
        match self {
            Self::Second => 's',
            Self::Minute => 'm',
            Self::Hour => 'h',
            Self::Day => 'd',
        }
    }
}

/// A wait written the way an operator thinks about it: `30s`, `15m`, `24h`, `7d`.
///
/// Hand-parsed rather than taken from jiff, which refuses `7d` in a duration
/// because a calendar day is not always 24 hours. That distinction is real and
/// jiff is right to draw it, but a backup cadence is a stopwatch rather than a
/// calendar: `7d` here means 168 hours, in every timezone and across every
/// daylight-saving boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Interval(Duration);

impl Interval {
    /// Builds an interval from whole seconds.
    pub const fn from_secs(seconds: u64) -> Self {
        Self(Duration::from_secs(seconds))
    }

    /// The wait, for handing to a timer.
    pub const fn as_duration(self) -> Duration {
        self.0
    }

    /// The wait in whole seconds.
    pub const fn as_secs(self) -> u64 {
        self.0.as_secs()
    }

    /// This interval multiplied by `factor`, for budgets stated as a multiple of
    /// a cadence ("stale after two intervals").
    pub const fn times(self, factor: u32) -> Self {
        Self(Duration::from_secs(
            self.0.as_secs().saturating_mul(factor as u64),
        ))
    }

    /// True when this interval is zero, which every caller reads as "disabled"
    /// rather than "run continuously".
    pub const fn is_disabled(self) -> bool {
        self.0.as_secs() == 0
    }
}

impl FromStr for Interval {
    type Err = IntervalError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let suffix = trimmed.chars().last().ok_or(IntervalError::Empty)?;
        let unit = IntervalUnit::from_suffix(suffix).ok_or_else(|| IntervalError::UnknownUnit {
            text: trimmed.to_owned(),
        })?;

        let digits = &trimmed[..trimmed.len() - suffix.len_utf8()];
        let amount: u64 = digits
            .trim()
            .parse()
            .map_err(|_| IntervalError::NotANumber {
                text: trimmed.to_owned(),
            })?;

        Ok(Self::from_secs(amount.saturating_mul(unit.seconds())))
    }
}

impl fmt::Display for Interval {
    /// Writes the largest unit that divides the wait exactly, so `86400s` and
    /// `24h` both read back as `1d`. This is a canonical spelling rather than
    /// the one an operator typed, which is what makes two configs comparable.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.as_secs();
        let unit = [
            IntervalUnit::Day,
            IntervalUnit::Hour,
            IntervalUnit::Minute,
            IntervalUnit::Second,
        ]
        .into_iter()
        .find(|candidate| seconds.is_multiple_of(candidate.seconds()))
        .unwrap_or(IntervalUnit::Second);

        write!(formatter, "{}{}", seconds / unit.seconds(), unit.suffix())
    }
}

#[cfg(test)]
#[path = "interval_test.rs"]
mod interval_test;
