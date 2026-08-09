//! Configuration: the non-secret settings and the credentials, each read from
//! its own kind of source and joined into [`Settings`].
//!
//! A [`ConfigSource`] supplies the non-secret half from one place, a file or the
//! environment. A [`SecretSource`] supplies the credentials. They never fill
//! each other's gaps, so a leaked config source carries no token and a new
//! backend is one more implementation.

mod core;
mod database;
mod env;
mod env_secret_source;
mod env_source;
mod errors;
mod schedule;
mod secrets;
mod settings;
mod source;
mod toml_source;
mod walg;

pub use core::ConfigCore;
pub use database::{ContainerSource, DatabaseSettings};
pub use env_secret_source::EnvSecretSource;
pub use env_source::EnvSource;
pub use errors::ConfigError;
pub use schedule::ScheduleSettings;
pub use secrets::{Secrets, StorageCredentials, WalgCredentials};
pub use settings::{Settings, StorageSettings};
pub use source::{ConfigSource, SecretSource};
pub use toml_source::{CONFIG_FILENAME, TomlSource};
pub use walg::{WalgMode, WalgSettings};

/// Region label for services that ignore regions, R2 among them.
pub const DEFAULT_REGION: &str = "auto";

/// The wal-g binary, as it is normally installed.
pub const DEFAULT_WALG_BINARY: &str = "wal-g";

/// The region a `[storage]` or `[walg]` table takes when it names none. A
/// single definition so the file and environment sources cannot disagree.
pub(crate) fn default_region() -> String {
    DEFAULT_REGION.to_owned()
}

/// Serialises the tests that read credentials.
///
/// The credential variables are process-global, so two test files guarding them
/// with two different mutexes do not exclude each other: one clears what the
/// other just set, and the failure lands on whichever variable lost the race.
/// One lock, shared here, is what makes them take turns.
#[cfg(test)]
pub(crate) static ENV_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());
