//! Reads a finished archive back with `pg_restore --list`.
//!
//! The check runs in a throwaway container built from the configured image, not
//! in the source container. Copying a multi-gigabyte archive into a live
//! database's filesystem to inspect it is a side effect on production that this
//! tool has no business causing.
//!
//! A host bind-mount would avoid the copy, but Docker Desktop only shares
//! configured paths, so a temporary working directory cannot be mounted
//! portably. `docker cp` into a container this tool owns works everywhere.

use std::path::Path;

use super::DockerError;
use super::container::{DockerSubcommand, NAME_FLAG, remove, run_docker};
use super::postgres_cli::{PostgresTool, copy_into};

/// Name of the container this inspection runs in.
const INSPECT_CONTAINER: &str = "backito-inspect";

/// Path the archive is copied to inside that container.
const ARCHIVE_PATH: &str = "/tmp/archive.dump";

/// How long the container stays alive waiting for the inspection, in seconds.
/// Long enough to copy a large archive, short enough that a crashed run leaves
/// nothing behind for more than a few minutes.
const CONTAINER_LIFETIME: &str = "600";

/// Counts `TABLE DATA` entries in `archive`, which is how many tables carry
/// rows.
///
/// A dump cut short by a full disk, a killed container, or a redirect that
/// never received bytes fails here, before it can be uploaded and mistaken for
/// a good backup.
pub async fn count_table_data_entries(image: &str, archive: &Path) -> Result<usize, DockerError> {
    remove(INSPECT_CONTAINER).await?;
    start_idle_container(image).await?;

    let counted = inspect(archive).await;

    remove(INSPECT_CONTAINER).await?;
    counted
}

/// Starts a container that does nothing but stay alive, so files can be copied
/// into it and `pg_restore` can be executed against them.
async fn start_idle_container(image: &str) -> Result<(), DockerError> {
    let subcommand = DockerSubcommand::Run.as_arg();
    run_docker(
        subcommand,
        &[
            subcommand,
            "-d",
            NAME_FLAG,
            INSPECT_CONTAINER,
            "--entrypoint",
            "sleep",
            image,
            CONTAINER_LIFETIME,
        ],
    )
    .await?;
    Ok(())
}

/// Copies the archive in and lists it.
async fn inspect(archive: &Path) -> Result<usize, DockerError> {
    copy_into(INSPECT_CONTAINER, archive, ARCHIVE_PATH).await?;

    let tool = PostgresTool::Restore;
    let listing = run_docker(
        tool.as_arg(),
        &[
            DockerSubcommand::Exec.as_arg(),
            INSPECT_CONTAINER,
            tool.as_arg(),
            "--list",
            ARCHIVE_PATH,
        ],
    )
    .await?;

    Ok(count_entries(&String::from_utf8_lossy(&listing)))
}

/// Counts the `TABLE DATA` lines in a `pg_restore --list` listing.
pub fn count_entries(listing: &str) -> usize {
    listing
        .lines()
        .filter(|line| line.contains("TABLE DATA"))
        .count()
}

#[cfg(test)]
#[path = "inspect_archive_test.rs"]
mod inspect_archive_test;
