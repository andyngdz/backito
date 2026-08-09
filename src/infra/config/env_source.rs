//! Reads the non-secret settings from `BACKITO_*` environment variables.
//!
//! The whole configuration comes from the environment here, not just the pieces
//! a file leaves out. That is the point of a source: pick this one and every
//! non-secret value is an env var, so a container needs no committed file and no
//! endpoint baked into an image.

use super::database::{
    default_container_label, default_restore_jobs, default_user, resolve_container,
};
use super::schedule::{DEFAULT_BACKUP_INTERVAL, DEFAULT_RETAIN, DEFAULT_VERIFY_INTERVAL};
use super::walg::{
    default_base_interval, default_binary, default_data_dir, default_retain_full,
    parse_base_interval,
};
use super::{
    ConfigCore, ConfigError, ConfigSource, DatabaseSettings, ScheduleSettings, StorageSettings,
    WalgMode, WalgSettings, default_region, env,
};

/// Reads settings from `BACKITO_*` variables.
pub struct EnvSource;

impl ConfigSource for EnvSource {
    fn load(&self) -> Result<ConfigCore, ConfigError> {
        let storage = read_storage()?;
        let walg = read_walg(&storage.endpoint)?;

        Ok(ConfigCore {
            database: read_database()?,
            storage,
            schedule: read_schedule()?,
            walg,
        })
    }
}

/// Reads the `[database]` fields, settling container against service.
fn read_database() -> Result<DatabaseSettings, ConfigError> {
    let container_label =
        env::optional("BACKITO_DB_CONTAINER_LABEL").unwrap_or_else(default_container_label);
    Ok(DatabaseSettings {
        label: env::required("BACKITO_DB_LABEL")?,
        container: resolve_container(
            env::optional("BACKITO_DB_CONTAINER"),
            env::optional("BACKITO_DB_SERVICE"),
            container_label,
        )?,
        name: env::required("BACKITO_DB_NAME")?,
        user: env::optional("BACKITO_DB_USER").unwrap_or_else(default_user),
        image: env::required("BACKITO_DB_IMAGE")?,
        restore_jobs: env::parse_or("BACKITO_DB_RESTORE_JOBS", default_restore_jobs())?,
    })
}

/// Reads the `[storage]` fields. The endpoint is required, the same as it is in
/// a file: a source that omits it is incomplete.
fn read_storage() -> Result<StorageSettings, ConfigError> {
    Ok(StorageSettings {
        endpoint: env::required("BACKITO_ENDPOINT")?,
        bucket: env::required("BACKITO_BUCKET")?,
        region: env::optional("BACKITO_REGION").unwrap_or_else(default_region),
    })
}

/// Reads the `[schedule]` fields, each defaulted the same as in a file.
fn read_schedule() -> Result<ScheduleSettings, ConfigError> {
    Ok(ScheduleSettings {
        backup_interval: env::parse_or("BACKITO_BACKUP_INTERVAL", DEFAULT_BACKUP_INTERVAL)?,
        verify_interval: env::parse_or("BACKITO_VERIFY_INTERVAL", DEFAULT_VERIFY_INTERVAL)?,
        retain: env::parse_or("BACKITO_RETAIN", DEFAULT_RETAIN)?,
    })
}

/// Reads the `[walg]` fields. `BACKITO_WALG_S3_PREFIX` decides the mode: set it
/// to archive WAL, leave it out to take only logical backups.
fn read_walg(storage_endpoint: &str) -> Result<WalgMode, ConfigError> {
    let Some(s3_prefix) = env::optional("BACKITO_WALG_S3_PREFIX") else {
        return Ok(WalgMode::Disabled);
    };

    let base_interval = match env::optional("BACKITO_WALG_BASE_INTERVAL") {
        Some(text) => parse_base_interval(&text)?,
        None => parse_base_interval(&default_base_interval())?,
    };

    Ok(WalgMode::Enabled(Box::new(WalgSettings {
        s3_prefix,
        endpoint: env::optional("BACKITO_WALG_ENDPOINT")
            .unwrap_or_else(|| storage_endpoint.to_owned()),
        region: env::optional("BACKITO_WALG_REGION").unwrap_or_else(default_region),
        data_dir: env::optional("BACKITO_WALG_DATA_DIR").unwrap_or_else(default_data_dir),
        base_interval,
        retain_full: env::parse_or("BACKITO_WALG_RETAIN_FULL", default_retain_full())?,
        binary: env::optional("BACKITO_WALG_BINARY").unwrap_or_else(default_binary),
    })))
}

#[cfg(test)]
#[path = "env_source_test.rs"]
mod env_source_test;
