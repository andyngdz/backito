//! Configuration loading: `backito.toml` plus credentials from the environment.

mod database;
mod errors;
mod schedule;
mod settings;
mod walg;

pub use database::{ContainerSource, DatabaseSettings};
pub use errors::ConfigError;
pub use schedule::ScheduleSettings;
pub use settings::{CONFIG_FILENAME, Settings, StorageCredentials, StorageSettings};
pub use walg::{WalgCredentials, WalgMode, WalgSettings};

/// Region label for services that ignore regions, R2 among them.
pub const DEFAULT_REGION: &str = "auto";

/// The wal-g binary, as it is normally installed.
pub const DEFAULT_WALG_BINARY: &str = "wal-g";

/// Serialises the tests that read credentials.
///
/// The credential variables are process-global, so two test files guarding them
/// with two different mutexes do not exclude each other: one clears what the
/// other just set, and the failure lands on whichever variable lost the race.
/// One lock, shared here, is what makes them take turns.
#[cfg(test)]
pub(crate) static ENV_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());
