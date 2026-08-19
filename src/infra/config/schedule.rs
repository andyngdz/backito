//! The `[schedule]` table: how often the long-running commands do their work.

use serde::Deserialize;

use super::ConfigError;
use crate::domain::Interval;

/// How often the scheduled commands do their work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleSettings {
    /// Wait between backups. `daemon` skips a pass when the newest archive is
    /// younger than this, so a restarted container does not dump again.
    pub backup_interval: Interval,
    /// Wait between restore-and-compare runs. Zero disables verification.
    pub verify_interval: Interval,
    /// Archives kept per label. Older ones are deleted after a backup lands.
    pub retain: u32,
}

/// The `[schedule]` table as written, with every interval still text.
///
/// Intervals arrive as `24h` rather than a count of seconds, so the unit is
/// visible in the file instead of living in a comment beside `86400`.
#[derive(Debug, Deserialize)]
pub struct ScheduleFile {
    #[serde(default = "default_backup_interval")]
    backup_interval: String,
    #[serde(default = "default_verify_interval")]
    verify_interval: String,
    #[serde(default = "default_retain")]
    retain: u32,
}

impl Default for ScheduleFile {
    /// Used when the file carries no `[schedule]` table at all, which is the
    /// normal case for a project that only ever runs one-shot commands.
    fn default() -> Self {
        Self {
            backup_interval: default_backup_interval(),
            verify_interval: default_verify_interval(),
            retain: default_retain(),
        }
    }
}

impl ScheduleFile {
    /// Reads each interval, naming the field that would not parse.
    pub fn into_settings(self) -> Result<ScheduleSettings, ConfigError> {
        Ok(ScheduleSettings {
            backup_interval: parse_interval("backup_interval", &self.backup_interval)?,
            verify_interval: parse_interval("verify_interval", &self.verify_interval)?,
            retain: checked_retain(self.retain)?,
        })
    }
}

/// Accepts a retention count only when it keeps something.
///
/// Both config sources funnel through here so a bucket-emptying `0` is refused
/// at the point it enters the process, rather than at the point it would delete.
/// A `verify_interval` of zero is a documented way to switch verification off;
/// retention has no such reading, because nothing survives it.
pub fn checked_retain(retain: u32) -> Result<u32, ConfigError> {
    match retain {
        0 => Err(ConfigError::RetainsNothing),
        kept => Ok(kept),
    }
}

/// Reads one interval field, keeping the field name in the failure so the
/// message points at the line to fix.
fn parse_interval(field: &str, text: &str) -> Result<Interval, ConfigError> {
    text.parse().map_err(|source| ConfigError::ParseInterval {
        field: field.to_owned(),
        source,
    })
}

/// One backup a day, the cadence a logical dump is sized for.
pub const DEFAULT_BACKUP_INTERVAL: Interval = Interval::from_secs(24 * 60 * 60);

/// One verification a week. A restore-and-compare costs a full download and a
/// full restore, so it runs far less often than the backup it checks.
pub const DEFAULT_VERIFY_INTERVAL: Interval = Interval::from_secs(7 * 24 * 60 * 60);

/// A month of daily archives.
///
/// Counted in archives rather than days, so it means a month only at the
/// default daily cadence. Corruption that goes unnoticed for a fortnight still
/// has a clean copy behind it here, which a one-week window would not.
pub const DEFAULT_RETAIN: u32 = 30;

impl Default for ScheduleSettings {
    fn default() -> Self {
        Self {
            backup_interval: DEFAULT_BACKUP_INTERVAL,
            verify_interval: DEFAULT_VERIFY_INTERVAL,
            retain: DEFAULT_RETAIN,
        }
    }
}

// The serde defaults are written from the same constants rather than as their
// own literals, so the value a missing field takes and the value `Default`
// hands back cannot drift apart.
fn default_backup_interval() -> String {
    DEFAULT_BACKUP_INTERVAL.to_string()
}

fn default_verify_interval() -> String {
    DEFAULT_VERIFY_INTERVAL.to_string()
}

fn default_retain() -> u32 {
    DEFAULT_RETAIN
}

#[cfg(test)]
#[path = "schedule_test.rs"]
mod schedule_test;
