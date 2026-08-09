//! The assembled configuration: the non-secret settings joined with the
//! credentials, one from a config source and one from a secret source.

use super::{
    ConfigCore, ConfigError, ConfigSource, DatabaseSettings, ScheduleSettings, SecretSource,
    Secrets, StorageCredentials, WalgCredentials, WalgMode, WalgSettings,
};

/// The whole configuration a command needs.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Which database to dump and where it runs.
    pub database: DatabaseSettings,
    /// Where archives are stored.
    pub storage: StorageSettings,
    /// Credentials for `storage`.
    pub credentials: StorageCredentials,
    /// Cadence for the long-running commands.
    pub schedule: ScheduleSettings,
    /// Physical backups through `wal-g`, or a named absence.
    pub walg: WalgMode,
    /// Credentials for WAL storage, present exactly when `walg` is enabled.
    /// [`Settings::from_parts`] is what establishes that pairing; read the two
    /// back through [`Settings::walg_runtime`] rather than separately, so a
    /// caller cannot hold settings whose token it never checked for.
    pub walg_credentials: Option<WalgCredentials>,
}

/// The storage half of the config, minus the token.
#[derive(Debug, Clone)]
pub struct StorageSettings {
    /// S3-compatible endpoint, e.g. `https://<account>.r2.cloudflarestorage.com`.
    pub endpoint: String,
    /// Bucket that holds archives. Nothing else should write to it.
    pub bucket: String,
    /// Region label. S3-compatible services that ignore regions want `auto`.
    pub region: String,
}

impl Settings {
    /// Loads the non-secret settings from `config` and the credentials from
    /// `secrets`, then joins them.
    pub fn load(
        config: &dyn ConfigSource,
        secrets: &dyn SecretSource,
    ) -> Result<Self, ConfigError> {
        Self::from_parts(config.load()?, secrets.load()?)
    }

    /// Joins a non-secret core with its credentials, requiring a WAL token
    /// exactly when the core archives WAL.
    pub fn from_parts(core: ConfigCore, secrets: Secrets) -> Result<Self, ConfigError> {
        let walg_credentials = match (&core.walg, secrets.walg) {
            (WalgMode::Enabled(_), None) => return Err(ConfigError::MissingWalgCredentials),
            (WalgMode::Enabled(_), Some(credentials)) => Some(credentials),
            (WalgMode::Disabled, _) => None,
        };
        Ok(Self {
            database: core.database,
            storage: core.storage,
            credentials: secrets.storage,
            schedule: core.schedule,
            walg: core.walg,
            walg_credentials,
        })
    }

    /// The WAL settings and their token together, when WAL archiving is on. The
    /// pair is what the `walg` commands need, and either both are present or
    /// neither is, so they are handed out as one.
    pub fn walg_runtime(&self) -> Option<(&WalgSettings, &WalgCredentials)> {
        match (&self.walg, &self.walg_credentials) {
            (WalgMode::Enabled(settings), Some(credentials)) => Some((settings, credentials)),
            (WalgMode::Enabled(_), None) => None,
            (WalgMode::Disabled, _) => None,
        }
    }
}

#[cfg(test)]
#[path = "settings_test.rs"]
mod settings_test;
