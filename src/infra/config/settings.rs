//! Loads `backito.toml` and the credentials that must never live in it.
//!
//! The file describes WHICH database and bucket a project backs up; it is meant
//! to be committed. Credentials come from the environment, so the same file
//! works under dotenvx, a CI secret store, or a plain export.

use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::ConfigError;
use super::database::{DatabaseFile, DatabaseSettings};
use super::schedule::{ScheduleFile, ScheduleSettings};
use super::walg::{WalgFile, WalgMode};

/// Default config filename looked up in the working directory.
pub const CONFIG_FILENAME: &str = "backito.toml";

/// Environment variable holding the S3 access key id.
const ACCESS_KEY_VAR: &str = "BACKITO_ACCESS_KEY_ID";

/// Environment variable holding the S3 secret access key.
const SECRET_KEY_VAR: &str = "BACKITO_SECRET_ACCESS_KEY";

/// Environment variable holding the S3 endpoint, used when the config omits it.
///
/// The endpoint carries the account id, so a project that commits its config
/// keeps the endpoint out of it and supplies this variable instead.
const ENDPOINT_VAR: &str = "BACKITO_ENDPOINT";

/// The whole configuration a command needs.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Which database to dump and where it runs.
    pub database: DatabaseSettings,
    /// Where archives are stored.
    pub storage: StorageSettings,
    /// Credentials for `storage`, read from the environment.
    pub credentials: StorageCredentials,
    /// Cadence for the long-running commands. Defaulted in full, so a config
    /// written for one-shot `backup` needs no `[schedule]` table at all.
    pub schedule: ScheduleSettings,
    /// Physical backups through `wal-g`, or a named absence when the config
    /// carries no `[walg]` table.
    pub walg: WalgMode,
}

/// The storage half of `backito.toml`, with the endpoint resolved.
#[derive(Debug, Clone)]
pub struct StorageSettings {
    /// S3-compatible endpoint, e.g. `https://<account>.r2.cloudflarestorage.com`.
    pub endpoint: String,
    /// Bucket that holds archives. Nothing else should write to it.
    pub bucket: String,
    /// Region label. S3-compatible services that ignore regions want `auto`.
    pub region: String,
}

/// The `[storage]` table as written, before the endpoint is resolved.
#[derive(Debug, Deserialize)]
struct StorageFile {
    endpoint: Option<String>,
    bucket: String,
    #[serde(default = "default_region")]
    region: String,
}

impl StorageFile {
    /// Resolves the endpoint from the file or `BACKITO_ENDPOINT`, treating a
    /// blank file value as absent so a committed config can omit it entirely.
    fn into_settings(self) -> Result<StorageSettings, ConfigError> {
        let endpoint = match self.endpoint {
            Some(value) if !value.trim().is_empty() => value,
            _ => endpoint_from_env()?,
        };
        Ok(StorageSettings {
            endpoint,
            bucket: self.bucket,
            region: self.region,
        })
    }
}

/// Credentials for the object store, sourced from the environment only.
#[derive(Debug, Clone)]
pub struct StorageCredentials {
    /// S3 access key id.
    pub access_key_id: String,
    /// S3 secret access key.
    pub secret_access_key: String,
}

/// `backito.toml` as parsed, before environment credentials are attached.
#[derive(Debug, Deserialize)]
struct SettingsFile {
    database: DatabaseFile,
    storage: StorageFile,
    #[serde(default)]
    schedule: ScheduleFile,
    walg: Option<WalgFile>,
}

fn default_region() -> String {
    super::DEFAULT_REGION.to_owned()
}

impl Settings {
    /// Loads settings from `path`, or from `backito.toml` in the working
    /// directory when `path` is `None`.
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        let config_path = path.map_or_else(|| PathBuf::from(CONFIG_FILENAME), Path::to_path_buf);
        let body =
            std::fs::read_to_string(&config_path).map_err(|source| ConfigError::ReadFile {
                path: config_path.clone(),
                source,
            })?;
        let parsed: SettingsFile =
            toml::from_str(&body).map_err(|source| ConfigError::ParseFile {
                path: config_path,
                source,
            })?;

        let storage = parsed.storage.into_settings()?;
        let storage_endpoint = storage.endpoint.clone();

        Ok(Self {
            database: parsed.database.into_settings()?,
            storage,
            credentials: StorageCredentials::from_env()?,
            schedule: parsed.schedule.into_settings()?,
            walg: match parsed.walg {
                Some(file) => WalgMode::Enabled(Box::new(file.into_settings(&storage_endpoint)?)),
                None => WalgMode::Disabled,
            },
        })
    }
}

/// Reads `BACKITO_ENDPOINT`, treating empty as absent so a blank export fails at
/// startup rather than as a connection error mid-upload.
fn endpoint_from_env() -> Result<String, ConfigError> {
    std::env::var(ENDPOINT_VAR)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigError::MissingEndpoint)
}

impl StorageCredentials {
    /// Reads both credential variables, naming the missing one on failure.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            access_key_id: required_var(ACCESS_KEY_VAR)?,
            secret_access_key: required_var(SECRET_KEY_VAR)?,
        })
    }
}

/// Reads `name` from the environment, treating empty as absent so a blank
/// export fails at startup instead of as a 403 mid-upload.
fn required_var(name: &str) -> Result<String, ConfigError> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ConfigError::MissingCredential {
            variable: name.to_owned(),
        })
}

#[cfg(test)]
#[path = "settings_test.rs"]
mod settings_test;

#[cfg(test)]
#[path = "settings_endpoint_test.rs"]
mod settings_endpoint_test;
