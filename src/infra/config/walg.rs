//! The `[walg]` table: physical backups, delegated to the `wal-g` binary.
//!
//! Logical and physical backups answer different questions. A `pg_dump` archive
//! restores the data as of the moment it was taken; WAL archiving plus a base
//! backup restores to any point between them. backito takes the first itself and
//! drives `wal-g` for the second rather than reimplementing it.
//!
//! Nothing here is secret. The WAL credentials live in [`super::Secrets`], read
//! by a secret source and attached when the settings are assembled, so a config
//! source can never carry them.

use serde::Deserialize;

use super::ConfigError;
use super::default_region;
use crate::domain::Interval;

/// Whether this project archives WAL at all.
///
/// An enum rather than an `Option`, because "no `[walg]` table" is a decision
/// with a name: a project that takes only logical backups. The commands that
/// need it can then say so instead of unwrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalgMode {
    /// No `[walg]` table. `walg` commands report this rather than guessing.
    Disabled,
    /// WAL archiving is configured.
    Enabled(Box<WalgSettings>),
}

/// What `wal-g` needs, and how often to give it a base backup. All non-secret:
/// the credentials for `s3_prefix` are held separately in [`super::Secrets`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalgSettings {
    /// Where WAL segments and base backups go, e.g. `s3://app-walg/`.
    ///
    /// This has to be a prefix no other cluster writes to. WAL segments are
    /// named after the LSN, which two clusters both produce, so sharing one
    /// prefix means each overwrites the other's archive.
    pub s3_prefix: String,
    /// S3-compatible endpoint. Defaults to `[storage].endpoint`, since the
    /// endpoint is account-level and only the bucket and its token differ.
    pub endpoint: String,
    /// Region label. Services that ignore regions want `auto`.
    pub region: String,
    /// The cluster's data directory, as seen from inside this container.
    pub data_dir: String,
    /// Wait between base backups.
    pub base_interval: Interval,
    /// Base backups kept. Older ones are deleted after a new one lands, along
    /// with the WAL segments they were the only reason to keep.
    pub retain_full: u32,
    /// The `wal-g` binary to drive.
    pub binary: String,
}

/// The `[walg]` table as written.
#[derive(Debug, Deserialize)]
pub struct WalgFile {
    s3_prefix: String,
    endpoint: Option<String>,
    #[serde(default = "default_region")]
    region: String,
    #[serde(default = "default_data_dir")]
    data_dir: String,
    #[serde(default = "default_base_interval")]
    base_interval: String,
    #[serde(default = "default_retain_full")]
    retain_full: u32,
    #[serde(default = "default_binary")]
    binary: String,
}

impl WalgFile {
    /// Reads the table, falling back to the archive store's endpoint. This is a
    /// default within one source, not a cross-source fill: both values come
    /// from the same file.
    pub fn into_settings(self, storage_endpoint: &str) -> Result<WalgSettings, ConfigError> {
        Ok(WalgSettings {
            s3_prefix: self.s3_prefix,
            endpoint: self.endpoint.unwrap_or_else(|| storage_endpoint.to_owned()),
            region: self.region,
            data_dir: self.data_dir,
            base_interval: parse_base_interval(&self.base_interval)?,
            retain_full: self.retain_full,
            binary: self.binary,
        })
    }
}

/// Reads the base-backup interval, naming the field on failure. Shared by the
/// file and environment sources so both report the same field name.
pub fn parse_base_interval(text: &str) -> Result<Interval, ConfigError> {
    text.parse().map_err(|source| ConfigError::ParseInterval {
        field: "base_interval".to_owned(),
        source,
    })
}

/// Where the Postgres images this tool is used with keep their data.
pub fn default_data_dir() -> String {
    "/var/lib/postgresql/data".to_owned()
}

/// One base backup a day, matching the logical cadence.
pub fn default_base_interval() -> String {
    "24h".to_owned()
}

/// Three base backups. Each is a full copy of the cluster, so this is a disk
/// bill rather than a retention policy in the logical sense.
pub fn default_retain_full() -> u32 {
    3
}

/// The `wal-g` binary as it is normally installed.
pub fn default_binary() -> String {
    super::DEFAULT_WALG_BINARY.to_owned()
}

#[cfg(test)]
#[path = "walg_test.rs"]
mod walg_test;
