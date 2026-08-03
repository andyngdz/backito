//! Command-line grammar.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Back up a containerised Postgres database to S3-compatible storage, and
/// prove the archive restores.
#[derive(Debug, Parser)]
#[command(name = "backito", version, about, long_about = None)]
pub struct Cli {
    /// Path to the config file (default: ./backito.toml).
    #[arg(long, short, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Suppress progress output. Warnings and errors still print.
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Print internal logs to stderr, for diagnosing a failure.
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// The commands `backito` offers.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Dump the database and upload the archive.
    ///
    /// Prints the stored object key on stdout. Progress goes to stderr, so the
    /// key can be captured on its own.
    ///
    /// Examples:
    ///   backito backup
    ///   backito backup --keep --config prod.toml
    Backup {
        /// Keep the local archive after upload instead of deleting it.
        #[arg(long)]
        keep: bool,
    },

    /// Restore an archive into a throwaway database and compare row counts
    /// against the live source.
    ///
    /// Nothing outside the throwaway container is written. Exits non-zero when
    /// the restored copy does not match its source.
    ///
    /// Examples:
    ///   backito verify
    ///   backito verify --archive app-backup-20260803-0942.dump
    Verify {
        /// Archive key to verify (default: the newest in the bucket).
        #[arg(long, value_name = "KEY")]
        archive: Option<String>,
    },

    /// Load an archive into a real database. This writes over existing data.
    ///
    /// Refuses a target that already holds tables unless --force is passed.
    ///
    /// Examples:
    ///   backito restore --into-container app-db-new
    ///   backito restore --force
    Restore {
        /// Container to restore into (default: the configured database).
        #[arg(long, value_name = "NAME")]
        into_container: Option<String>,

        /// Archive key to restore (default: the newest in the bucket).
        #[arg(long, value_name = "KEY")]
        archive: Option<String>,

        /// Proceed even though the target already holds data.
        #[arg(long)]
        force: bool,
    },
}
