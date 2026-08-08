//! Loads `backito.toml` and the credentials that must never live in it.
//!
//! The file describes WHICH database and bucket a project backs up; it is meant
//! to be committed. Credentials come from the environment, so the same file
//! works under dotenvx, a CI secret store, or a plain export.

use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::ConfigError;

/// Default config filename looked up in the working directory.
pub const CONFIG_FILENAME: &str = "backito.toml";

/// Environment variable holding the S3 access key id.
const ACCESS_KEY_VAR: &str = "BACKITO_ACCESS_KEY_ID";

/// Environment variable holding the S3 secret access key.
const SECRET_KEY_VAR: &str = "BACKITO_SECRET_ACCESS_KEY";

/// The whole configuration a command needs.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Which database to dump and where it runs.
    pub database: DatabaseSettings,
    /// Where archives are stored.
    pub storage: StorageSettings,
    /// Credentials for `storage`, read from the environment.
    pub credentials: StorageCredentials,
}

/// The database half of `backito.toml`.
#[derive(Debug, Clone)]
pub struct DatabaseSettings {
    /// Short name used to build archive keys, e.g. `app`.
    pub label: String,
    /// How to find the container running Postgres. The dump runs inside it, so
    /// the `pg_dump` binary always matches the server version.
    pub container: ContainerSource,
    /// Database name to dump.
    pub name: String,
    /// Role to connect as. Inside the container this authenticates over the
    /// unix socket, so no password is needed.
    pub user: String,
    /// Container image used to build the throwaway database that `verify`
    /// restores into. Must be the same major version as `container` serves.
    pub image: String,
    /// Parallel jobs `pg_restore` uses when restoring. Parallel restore is
    /// faster, but each worker builds indexes in its own memory, so a
    /// memory-capped container can OOM mid-restore. Drop to 1 for tight targets.
    pub restore_jobs: u8,
}

/// How the running database container is identified.
///
/// A fixed name is the simplest thing that works and stays correct as long as
/// something guarantees the name. Nothing does under an orchestrator: compose
/// derives `<project>-<service>-<n>`, and uncloud appends a fresh random suffix
/// on every redeploy, so a long-running process holding a name captured at
/// startup ends up talking about a container that no longer exists. The service
/// name survives both, and both record it as a container label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerSource {
    /// A container name fixed in the config.
    Named(String),
    /// A service name, resolved through a container label on each use.
    Service {
        /// Label key carrying the service name.
        label: String,
        /// Service name to match.
        service: String,
    },
}

/// The storage half of `backito.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    /// S3-compatible endpoint, e.g. `https://<account>.r2.cloudflarestorage.com`.
    pub endpoint: String,
    /// Bucket that holds archives. Nothing else should write to it.
    pub bucket: String,
    /// Region label. S3-compatible services that ignore regions want `auto`.
    #[serde(default = "default_region")]
    pub region: String,
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
    storage: StorageSettings,
}

/// The `[database]` table exactly as it appears on disk.
///
/// Separate from `DatabaseSettings` because `container` and `service` are two
/// spellings of one choice on the way in, and a single settled `ContainerSource`
/// on the way out. Parsing is where that collapses, so no caller downstream has
/// to consider the pair that cannot both be set.
#[derive(Debug, Deserialize)]
struct DatabaseFile {
    label: String,
    container: Option<String>,
    service: Option<String>,
    #[serde(default = "default_container_label")]
    container_label: String,
    name: String,
    #[serde(default = "default_user")]
    user: String,
    image: String,
    #[serde(default = "default_restore_jobs")]
    restore_jobs: u8,
}

impl DatabaseFile {
    /// Settles `container` and `service` into one source, or explains why the
    /// pair as written cannot be honoured.
    fn into_settings(self) -> Result<DatabaseSettings, ConfigError> {
        let container = match (self.container, self.service) {
            (Some(_), Some(_)) => return Err(ConfigError::ContainerOverSpecified),
            (Some(name), None) => ContainerSource::Named(name),
            (None, Some(service)) => ContainerSource::Service {
                label: self.container_label,
                service,
            },
            (None, None) => return Err(ConfigError::ContainerUnspecified),
        };

        Ok(DatabaseSettings {
            label: self.label,
            container,
            name: self.name,
            user: self.user,
            image: self.image,
            restore_jobs: self.restore_jobs,
        })
    }
}

fn default_user() -> String {
    "postgres".to_owned()
}

fn default_region() -> String {
    "auto".to_owned()
}

/// Label key checked when the config names a service instead of a container.
/// Compose writes this one; uncloud writes `uncloud.service.name`, so a cluster
/// using it has to say so.
fn default_container_label() -> String {
    "com.docker.compose.service".to_owned()
}

/// Parallel restore jobs kept when the config omits `restore_jobs`. Matches the
/// value backito used before the knob existed.
fn default_restore_jobs() -> u8 {
    4
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

        Ok(Self {
            database: parsed.database.into_settings()?,
            storage: parsed.storage,
            credentials: StorageCredentials::from_env()?,
        })
    }
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
