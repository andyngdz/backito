//! Configuration loading: `backito.toml` plus credentials from the environment.

mod database;
mod errors;
mod schedule;
mod settings;

pub use database::{ContainerSource, DatabaseSettings};
pub use errors::ConfigError;
pub use schedule::ScheduleSettings;
pub use settings::{Settings, StorageCredentials, StorageSettings};
