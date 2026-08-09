//! The non-secret half of a configuration.
//!
//! A `ConfigSource` produces exactly this: which database and bucket a project
//! backs up, how often, and whether it archives WAL. No credentials, so the
//! same shape comes from a committed file or from the environment without either
//! carrying a token.

use super::{DatabaseSettings, ScheduleSettings, StorageSettings, WalgMode};

/// Everything a backup needs except the credentials.
#[derive(Debug, Clone)]
pub struct ConfigCore {
    /// Which database to dump and where it runs.
    pub database: DatabaseSettings,
    /// Where archives are stored, minus the token.
    pub storage: StorageSettings,
    /// Cadence for the long-running commands.
    pub schedule: ScheduleSettings,
    /// Physical backups through `wal-g`, or a named absence.
    pub walg: WalgMode,
}
