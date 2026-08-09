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
            retain: self.retain,
        })
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

/// A week of daily archives.
pub const DEFAULT_RETAIN: u32 = 7;

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
