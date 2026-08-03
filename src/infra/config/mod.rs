//! Configuration loading: `backito.toml` plus credentials from the environment.

mod errors;
mod settings;

pub use errors::ConfigError;
pub use settings::{DatabaseSettings, Settings, StorageCredentials, StorageSettings};
