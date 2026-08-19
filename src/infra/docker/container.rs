//! Thin wrapper over the `docker` CLI: run a command in a container, start a
//! throwaway one, remove it.
//!
//! Every argument is passed as a separate process argument -- no shell string is
//! ever assembled, so a container or database name cannot become shell syntax.

use std::process::Stdio;
use tokio::process::Command;

use super::DockerError;

/// The `docker` subcommands this tool drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerSubcommand {
    /// Start a container.
    Run,
    /// Remove a container.
    Rm,
    /// Execute a program inside a running container.
    Exec,
    /// List running containers.
    Ps,
}

impl DockerSubcommand {
    /// The literal argument passed to `docker`.
    pub fn as_arg(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Rm => "rm",
            Self::Exec => "exec",
            Self::Ps => "ps",
        }
    }
}

/// The `docker` binary this tool drives.
pub const DOCKER_BIN: &str = "docker";

/// Flag naming a container on `docker run`.
pub const NAME_FLAG: &str = "--name";

/// Narrows a `docker ps` listing.
const FILTER_FLAG: &str = "--filter";

/// Picks the one field a `docker ps` listing prints.
const FORMAT_FLAG: &str = "--format";

/// Template that prints just the container name, one per line.
const NAMES_TEMPLATE: &str = "{{.Names}}";

/// `pg_isready`, the readiness probe run inside a container.
const PG_ISREADY: &str = "pg_isready";

/// How long to wait for a freshly started container to accept connections.
const READY_ATTEMPTS: u32 = 60;

/// Delay between readiness probes.
const READY_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

/// Bytes of stderr kept in an error message.
const STDERR_KEEP_BYTES: usize = 2000;

/// Runs `docker` with `args`, returning stdout on success.
pub async fn run_docker(operation: &str, args: &[&str]) -> Result<Vec<u8>, DockerError> {
    let output = Command::new(DOCKER_BIN)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|source| DockerError::Spawn {
            operation: operation.to_owned(),
            source,
        })?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    Err(DockerError::Exit {
        operation: operation.to_owned(),
        status: output.status.to_string(),
        stderr: trailing_stderr(&output.stderr),
    })
}

/// Returns true when `container` exists and is running.
///
/// Asks `docker ps` rather than `docker inspect` so that an absent container and
/// an unusable docker are different answers. `inspect` exits non-zero for both,
/// and reading that as "not running" told the operator to go check a container
/// that was fine while the real fault was a dead socket or a permission error on
/// it. A listing exits zero whenever docker answered at all, so only a genuine
/// infrastructure failure propagates.
pub async fn is_running(container: &str) -> Result<bool, DockerError> {
    let subcommand = DockerSubcommand::Ps.as_arg();
    // Docker matches this filter as a regex, and a name may carry `.`, so the
    // filter narrows the listing and the exact comparison below decides.
    let filter = format!("name=^{container}$");
    let listed = run_docker(
        subcommand,
        &[
            subcommand,
            FILTER_FLAG,
            &filter,
            FORMAT_FLAG,
            NAMES_TEMPLATE,
        ],
    )
    .await?;

    Ok(String::from_utf8_lossy(&listed)
        .lines()
        .map(str::trim)
        .any(|name| name == container))
}

/// Names the running container carrying `label`=`service`.
///
/// Orchestrators that mint container names do not let you fix one: compose
/// derives `<project>-<service>-<n>`, and uncloud appends a fresh random suffix
/// on every redeploy. The service name is the part that stays put, and both
/// record it as a label, so that is what a long-running caller should resolve
/// against rather than a name captured once at startup.
///
/// Ties are broken by taking the first line. A service scaled past one replica
/// has several, and for a database any of them is the wrong thing to guess at,
/// so callers that care should pin `container` instead.
pub async fn resolve_by_label(label: &str, service: &str) -> Result<String, DockerError> {
    let subcommand = DockerSubcommand::Ps.as_arg();
    let filter = format!("label={label}={service}");
    let listed = run_docker(
        subcommand,
        &[
            subcommand,
            FILTER_FLAG,
            &filter,
            FORMAT_FLAG,
            NAMES_TEMPLATE,
        ],
    )
    .await?;

    String::from_utf8_lossy(&listed)
        .lines()
        .map(str::trim)
        .find(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| DockerError::NoContainerForService {
            label: label.to_owned(),
            service: service.to_owned(),
        })
}

/// Fails unless `container` is running, so a command stops before it does work
/// that cannot land.
pub async fn require_running(container: &str) -> Result<(), DockerError> {
    if is_running(container).await? {
        return Ok(());
    }
    Err(DockerError::ContainerNotRunning {
        container: container.to_owned(),
    })
}

/// Starts a throwaway Postgres container with no volume and no published port,
/// so removing it destroys everything it held and nothing on the host can
/// mistake it for a real database.
pub async fn start_throwaway(container: &str, image: &str) -> Result<(), DockerError> {
    remove(container).await?;
    let subcommand = DockerSubcommand::Run.as_arg();
    run_docker(
        subcommand,
        &[
            subcommand,
            "-d",
            NAME_FLAG,
            container,
            "-e",
            "POSTGRES_PASSWORD=throwaway",
            image,
        ],
    )
    .await?;
    Ok(())
}

/// Waits until `container` reports ready for connections.
pub async fn wait_ready(container: &str, user: &str) -> Result<(), DockerError> {
    for _ in 0..READY_ATTEMPTS {
        let probe = run_docker(
            PG_ISREADY,
            &[
                DockerSubcommand::Exec.as_arg(),
                container,
                PG_ISREADY,
                "-U",
                user,
                "-h",
                "localhost",
            ],
        )
        .await;
        if probe.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(READY_INTERVAL).await;
    }

    Err(DockerError::ContainerNotRunning {
        container: container.to_owned(),
    })
}

/// Removes `container`, treating "no such container" as already removed.
///
/// A non-zero exit stays a success because every caller wants the name free
/// rather than the command to have run, and "it was not there" is that. It is
/// logged rather than dropped: the same exit code also covers a container
/// another process holds, and a silent one there turns into a confusing
/// "name already in use" from the caller that follows.
pub async fn remove(container: &str) -> Result<(), DockerError> {
    let subcommand = DockerSubcommand::Rm.as_arg();
    match run_docker(subcommand, &[subcommand, "-f", container]).await {
        Ok(_) => Ok(()),
        Err(DockerError::Exit { stderr, .. }) => {
            tracing::debug!(container, %stderr, "docker rm reported nothing to remove");
            Ok(())
        }
        Err(other) => Err(other),
    }
}

/// Keeps the tail of stderr, which is where the useful line is when a tool
/// prints a long preamble first. Cuts on a char boundary so the message stays
/// valid UTF-8.
pub fn trailing_stderr(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let trimmed = text.trim();
    if trimmed.len() <= STDERR_KEEP_BYTES {
        return trimmed.to_owned();
    }

    let cut_from = trimmed.len() - STDERR_KEEP_BYTES;
    let boundary = (cut_from..trimmed.len())
        .find(|index| trimmed.is_char_boundary(*index))
        .unwrap_or(trimmed.len());
    trimmed[boundary..].to_owned()
}

#[cfg(test)]
#[path = "container_test.rs"]
mod container_test;
