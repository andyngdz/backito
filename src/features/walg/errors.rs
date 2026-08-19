//! Failures raised while driving wal-g.

use thiserror::Error;

/// Why a `walg` command could not do its work.
#[derive(Debug, Error)]
pub enum WalgError {
    /// The config carries no `[walg]` table.
    #[error("no [walg] section in the config, so there is nothing to archive WAL to")]
    NotConfigured,

    /// A program could not be executed at all.
    #[error("run {binary}: {source}")]
    Exec {
        /// The program that could not be started.
        binary: String,
        /// Underlying io failure.
        source: std::io::Error,
    },

    /// wal-g ran and exited non-zero.
    ///
    /// Carries the stderr tail because the scheduler retries this every fifteen
    /// minutes: without the reason, a base backup that will never succeed logs
    /// the same status line forever and says nothing about why.
    #[error("wal-g {operation} exited with status {status}: {stderr}")]
    Exit {
        /// What was being attempted, e.g. `backup-push`.
        operation: String,
        /// Exit status reported by wal-g.
        status: String,
        /// Tail of wal-g's stderr, where the cause is printed.
        stderr: String,
    },

    /// The Postgres configuration fragment could not be written.
    #[error("write {path}: {source}")]
    WriteConfig {
        /// Path that was attempted.
        path: String,
        /// Underlying io failure.
        source: std::io::Error,
    },
}
