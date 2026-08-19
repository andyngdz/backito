//! Adapters for everything outside this process: config files, the environment,
//! logging, the `docker` CLI, and S3-compatible storage.

pub mod config;
pub mod docker;
pub mod logging;
pub mod object_store;
pub mod shutdown;
pub mod workspace;
