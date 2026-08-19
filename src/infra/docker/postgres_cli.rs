//! Runs `pg_dump`, `pg_restore`, and `psql` inside a Postgres container.
//!
//! Running them in the container rather than on the host means the client
//! binaries always match the server version, and the host needs no Postgres
//! install at all.

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use super::DockerError;
use super::container::{DOCKER_BIN, DockerSubcommand, run_docker, trailing_stderr};
use super::count_query::{counts_sql, list_tables_sql, parse_counts, parse_identifiers};
use crate::domain::TableCounts;

/// The Postgres command-line tools this adapter drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostgresTool {
    /// Writes an archive.
    Dump,
    /// Reads an archive back.
    Restore,
    /// Runs SQL.
    Psql,
}

impl PostgresTool {
    /// The binary name inside the container.
    pub fn as_arg(self) -> &'static str {
        match self {
            Self::Dump => "pg_dump",
            Self::Restore => "pg_restore",
            Self::Psql => "psql",
        }
    }
}

/// Compression level for the custom-format archive. Level 3 is where the size
/// curve flattens; higher levels cost minutes for single-digit percentages.
const DUMP_COMPRESSION: &str = "--compress=3";

/// Custom format is required: it is the only one `pg_restore -j` can parallelise.
const DUMP_FORMAT: &str = "--format=custom";

/// Where an archive is staged inside a container before `pg_restore` reads it.
///
/// Shared by `verify` and `restore`, because it is the path `copy_into` writes
/// to and `restore_in_container` reads from: two copies of it could drift into
/// a restore that reads a file nothing wrote.
pub const ARCHIVE_IN_CONTAINER: &str = "/tmp/backito-restore.dump";

/// Ownership and ACL statements are dropped from both ends, so an archive
/// restores into a database whose roles differ from the source's.
const IDENTITY_FLAGS: [&str; 2] = ["--no-owner", "--no-acl"];

/// Which container, database, and role a Postgres command runs against.
#[derive(Debug, Clone)]
pub struct PostgresTarget {
    /// Container running the server.
    pub container: String,
    /// Database to operate on.
    pub database: String,
    /// Role to connect as. Inside the container this authenticates over the
    /// unix socket, so no password is involved.
    pub user: String,
}

impl PostgresTarget {
    /// The leading `docker exec <container> <tool> -U <user> -d <database>`
    /// arguments every command shares.
    pub(super) fn exec_prefix(&self, tool: PostgresTool) -> Vec<String> {
        vec![
            DockerSubcommand::Exec.as_arg().to_owned(),
            self.container.clone(),
            tool.as_arg().to_owned(),
            "-U".to_owned(),
            self.user.clone(),
            "-d".to_owned(),
            self.database.clone(),
        ]
    }
}

/// Streams `pg_dump` output straight into `destination`.
///
/// The archive never passes through this process's memory: docker writes to the
/// file handle directly, so a 10 GB database costs no extra RAM.
pub async fn dump_to_file(target: &PostgresTarget, destination: &Path) -> Result<(), DockerError> {
    let tool = PostgresTool::Dump;
    let file = std::fs::File::create(destination).map_err(|source| DockerError::Spawn {
        operation: tool.as_arg().to_owned(),
        source,
    })?;

    let mut args = target.exec_prefix(tool);
    args.push(DUMP_FORMAT.to_owned());
    args.push(DUMP_COMPRESSION.to_owned());
    args.extend(IDENTITY_FLAGS.iter().map(|flag| (*flag).to_owned()));

    // `Command::output()` would replace the stdout handle set below with a pipe
    // and buffer the whole archive in memory -- which silently left a zero-byte
    // file. `spawn` + `wait_with_output` keeps the redirect and still collects
    // stderr, because it only drains the handles that are actually pipes.
    let child = Command::new(DOCKER_BIN)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| DockerError::Spawn {
            operation: tool.as_arg().to_owned(),
            source,
        })?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|source| DockerError::Spawn {
            operation: tool.as_arg().to_owned(),
            source,
        })?;

    if output.status.success() {
        return Ok(());
    }
    Err(DockerError::Exit {
        operation: tool.as_arg().to_owned(),
        status: output.status.to_string(),
        stderr: trailing_stderr(&output.stderr),
    })
}

/// Copies a host file into a container.
pub async fn copy_into(
    container: &str,
    source: &Path,
    destination_in_container: &str,
) -> Result<(), DockerError> {
    let source = source.to_string_lossy().into_owned();
    let destination = format!("{container}:{destination_in_container}");
    run_docker("cp", &["cp", &source, &destination]).await?;
    Ok(())
}

/// Restores an archive already present inside the container, returning the
/// restore's stderr.
///
/// A non-zero exit is deliberately not an error here: managed Postgres images
/// report dozens of failures for system objects they already own, and none of
/// them mean application rows failed to land. Row counts decide success.
pub async fn restore_in_container(
    target: &PostgresTarget,
    archive_in_container: &str,
    jobs: u8,
) -> Result<String, DockerError> {
    let tool = PostgresTool::Restore;
    let mut args = target.exec_prefix(tool);
    args.extend(IDENTITY_FLAGS.iter().map(|flag| (*flag).to_owned()));
    args.push(format!("-j{jobs}"));
    args.push(archive_in_container.to_owned());

    let output = Command::new(DOCKER_BIN)
        .args(&args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|source| DockerError::Spawn {
            operation: tool.as_arg().to_owned(),
            source,
        })?;

    Ok(String::from_utf8_lossy(&output.stderr).into_owned())
}

/// Runs one SQL statement, returning tab-separated rows with no header.
pub async fn query(target: &PostgresTarget, sql: &str) -> Result<String, DockerError> {
    let mut args = target.exec_prefix(PostgresTool::Psql);
    args.push("-tAF\t".to_owned());
    args.push("-c".to_owned());
    args.push(sql.to_owned());

    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    let stdout = run_docker(PostgresTool::Psql.as_arg(), &borrowed).await?;
    Ok(String::from_utf8_lossy(&stdout).into_owned())
}

/// Counts every table in `schema`, in two round trips: list the tables, then
/// count them all in one statement.
pub async fn table_counts(
    target: &PostgresTarget,
    schema: &str,
) -> Result<TableCounts, DockerError> {
    let listed = query(target, &list_tables_sql(schema)).await?;
    let tables = parse_identifiers(&listed);

    match counts_sql(schema, &tables) {
        None => Ok(TableCounts::new()),
        Some(sql) => Ok(parse_counts(&query(target, &sql).await?)),
    }
}

#[cfg(test)]
#[path = "postgres_cli_test.rs"]
mod postgres_cli_test;
