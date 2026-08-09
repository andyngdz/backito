//! The `[database]` table: which database to dump, and which container runs it.

use serde::Deserialize;

use super::ConfigError;

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
/// A fixed name is the simplest thing that works, and stays correct as long as
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

/// The `[database]` table exactly as it appears on disk.
///
/// Separate from `DatabaseSettings` because `container` and `service` are two
/// spellings of one choice on the way in, and a single settled `ContainerSource`
/// on the way out. Parsing is where that collapses, so no caller downstream has
/// to consider the pair that cannot both be set.
#[derive(Debug, Deserialize)]
pub struct DatabaseFile {
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
    pub fn into_settings(self) -> Result<DatabaseSettings, ConfigError> {
        Ok(DatabaseSettings {
            label: self.label,
            container: resolve_container(self.container, self.service, self.container_label)?,
            name: self.name,
            user: self.user,
            image: self.image,
            restore_jobs: self.restore_jobs,
        })
    }
}

/// Settles the container/service pair into one source. Shared by the file and
/// environment sources so both refuse "both" and "neither" the same way.
pub fn resolve_container(
    container: Option<String>,
    service: Option<String>,
    container_label: String,
) -> Result<ContainerSource, ConfigError> {
    match (container, service) {
        (Some(_), Some(_)) => Err(ConfigError::ContainerOverSpecified),
        (Some(name), None) => Ok(ContainerSource::Named(name)),
        (None, Some(service)) => Ok(ContainerSource::Service {
            label: container_label,
            service,
        }),
        (None, None) => Err(ConfigError::ContainerUnspecified),
    }
}

/// Role connected as when the config names none. Inside the container this
/// authenticates over the unix socket, so no password is involved.
pub fn default_user() -> String {
    "postgres".to_owned()
}

/// Label key checked when the config names a service instead of a container.
/// Compose writes this one; uncloud writes `uncloud.service.name`, so a cluster
/// using it has to say so.
pub fn default_container_label() -> String {
    "com.docker.compose.service".to_owned()
}

/// Parallel restore jobs kept when the config omits `restore_jobs`. Matches the
/// value backito used before the knob existed.
pub fn default_restore_jobs() -> u8 {
    4
}
