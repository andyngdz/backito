//! Turning the configured container source into the name Docker answers to.
//!
//! Kept out of the config layer because resolving is a call to Docker, not a
//! property of the file, and out of the docker layer because the choice between
//! a pinned name and a labelled service is configuration. Every command that
//! touches the database goes through here, and a long-running one calls it again
//! on each pass rather than holding the answer.

use crate::infra::config::ContainerSource;
use crate::infra::docker::{DockerError, resolve_by_label};

/// Names the container to run Postgres commands in.
pub async fn resolve(source: &ContainerSource) -> Result<String, DockerError> {
    match source {
        ContainerSource::Named(name) => Ok(name.clone()),
        ContainerSource::Service { label, service } => resolve_by_label(label, service).await,
    }
}

#[cfg(test)]
#[path = "container_test.rs"]
mod container_test;
