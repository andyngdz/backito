//! Configuration loading: `backito.toml` plus credentials from the environment.

mod errors;
mod settings;

pub use errors::ConfigError;
pub use settings::{
    ContainerSource, DatabaseSettings, Settings, StorageCredentials, StorageSettings,
};
