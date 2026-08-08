//! Failures raised while driving Docker and the Postgres CLI tools.

use thiserror::Error;

/// Why a container command could not be run, or ran and failed.
#[derive(Debug, Error)]
pub enum DockerError {
    /// The `docker` binary could not be launched at all.
    #[error("run docker {operation}: {source}")]
    Spawn {
        /// What was being attempted, e.g. `pg_dump`.
        operation: String,
        /// Underlying io failure.
        source: std::io::Error,
    },

    /// The command ran and exited non-zero.
    #[error("docker {operation} exited with status {status}: {stderr}")]
    Exit {
        /// What was being attempted.
        operation: String,
        /// Exit status reported by docker.
        status: String,
        /// Trailing stderr, trimmed for the message.
        stderr: String,
    },

    /// The named container is not running.
    #[error("container {container} is not running")]
    ContainerNotRunning {
        /// Container that was expected to be up.
        container: String,
    },

    /// No running container carries the label the config named.
    #[error("no running container labelled {label}={service}")]
    NoContainerForService {
        /// Label key that was filtered on, e.g. `com.docker.compose.service`.
        label: String,
        /// Value that was expected, i.e. the service name.
        service: String,
    },
}
