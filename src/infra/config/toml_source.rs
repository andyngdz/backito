//! Reads the non-secret settings from a `backito.toml` file.
//!
//! The file describes which database and bucket a project backs up. It is meant
//! to be committed, so it holds no credentials. Because a source is read on its
//! own, the endpoint is a required field here rather than something the
//! environment fills in: a file that omits it is an incomplete file.

use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::database::DatabaseFile;
use super::schedule::ScheduleFile;
use super::walg::WalgFile;
use super::{ConfigCore, ConfigError, ConfigSource, StorageSettings, WalgMode, default_region};

/// Default config filename looked up in the working directory.
pub const CONFIG_FILENAME: &str = "backito.toml";

/// Reads settings from a TOML file.
pub struct TomlSource {
    path: PathBuf,
}

impl TomlSource {
    /// A source reading `path`, or `backito.toml` in the working directory when
    /// `path` is `None`.
    pub fn new(path: Option<&Path>) -> Self {
        Self {
            path: path.map_or_else(|| PathBuf::from(CONFIG_FILENAME), Path::to_path_buf),
        }
    }
}

impl ConfigSource for TomlSource {
    fn load(&self) -> Result<ConfigCore, ConfigError> {
        let body = std::fs::read_to_string(&self.path).map_err(|source| ConfigError::ReadFile {
            path: self.path.clone(),
            source,
        })?;
        let parsed: SettingsFile =
            toml::from_str(&body).map_err(|source| ConfigError::ParseFile {
                path: self.path.clone(),
                source,
            })?;

        let storage = parsed.storage.into_settings();
        let walg = match parsed.walg {
            Some(file) => WalgMode::Enabled(Box::new(file.into_settings(&storage.endpoint)?)),
            None => WalgMode::Disabled,
        };

        Ok(ConfigCore {
            database: parsed.database.into_settings()?,
            storage,
            schedule: parsed.schedule.into_settings()?,
            walg,
        })
    }
}

/// `backito.toml` as parsed.
#[derive(Debug, Deserialize)]
struct SettingsFile {
    database: DatabaseFile,
    storage: StorageFile,
    #[serde(default)]
    schedule: ScheduleFile,
    walg: Option<WalgFile>,
}

/// The `[storage]` table as written. Endpoint is required: a committed file that
/// wants the endpoint kept out of it belongs to the environment source instead.
#[derive(Debug, Deserialize)]
struct StorageFile {
    endpoint: String,
    bucket: String,
    #[serde(default = "default_region")]
    region: String,
}

impl StorageFile {
    fn into_settings(self) -> StorageSettings {
        StorageSettings {
            endpoint: self.endpoint,
            bucket: self.bucket,
            region: self.region,
        }
    }
}

#[cfg(test)]
#[path = "toml_source_test.rs"]
mod toml_source_test;
