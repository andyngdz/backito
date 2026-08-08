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
    /// Write a starter backito.toml here and keep it out of git.
    ///
    /// Run this once per project, then open the file and fill in the endpoint
    /// and bucket. The config names a bucket and an S3 endpoint, so it is added
    /// to .gitignore rather than committed.
    ///
    /// Examples:
    ///   backito init
    ///   backito init --force
    Init {
        /// Replace an existing backito.toml instead of refusing.
        #[arg(long)]
        force: bool,
    },

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

    /// Back up on a schedule until stopped, pruning and verifying as configured.
    ///
    /// Reads its cadence from [schedule] in the config. On start it asks the
    /// bucket when the last backup landed and waits out the remainder, so a
    /// restarted container does not dump the database again.
    ///
    /// Examples:
    ///   backito daemon
    ///   backito daemon --config prod.toml
    Daemon,

    /// Report whether a recent enough backup exists. Exits 1 when none does.
    ///
    /// Built for a container healthcheck. It asks the bucket rather than a local
    /// marker file, so a restarted container cannot report itself healthy just
    /// by having forgotten. A backup is recent enough while it is younger than
    /// two [schedule] backup_interval periods.
    ///
    /// Examples:
    ///   backito health
    Health,

    /// Physical backups, driven through the `wal-g` binary.
    ///
    /// Reads the [walg] section. Without one, `archive` exits 0 so a container
    /// with no WAL storage recycles its segments normally, while `base` says
    /// there is nothing configured.
    #[command(subcommand)]
    Walg(WalgCommand),
}

/// The `walg` subcommands.
#[derive(Debug, Subcommand)]
pub enum WalgCommand {
    /// Push one WAL segment. Postgres runs this as archive_command.
    ///
    /// Examples:
    ///   backito walg archive /var/lib/postgresql/data/pg_wal/000000010000000000000001
    Archive {
        /// Path to the completed segment, which Postgres passes as %p.
        #[arg(value_name = "SEGMENT")]
        segment: String,
    },

    /// Take base backups on a cadence until stopped.
    ///
    /// WAL segments restore nothing on their own: they have to be replayed onto
    /// a base backup from the same cluster. On start this asks wal-g when the
    /// last one landed, so a restarted container does not take another.
    ///
    /// Examples:
    ///   backito walg base
    Base,

    /// Write Postgres archiving settings, then exec the image's entrypoint.
    ///
    /// Meant as a container ENTRYPOINT. It replaces itself with the program
    /// that follows, so Postgres keeps PID 1 and signals reach it unchanged.
    ///
    /// Examples:
    ///   backito walg entrypoint --fragment /etc/postgresql-custom/wal-g.conf -- docker-entrypoint.sh postgres
    Entrypoint {
        /// File to write the archiving settings into. The image decides where
        /// its Postgres reads extra configuration from.
        #[arg(long, value_name = "FILE")]
        fragment: PathBuf,

        /// The program to hand over to.
        #[arg(value_name = "PROGRAM")]
        program: String,

        /// Arguments for that program.
        #[arg(trailing_var_arg = true, value_name = "ARGS")]
        args: Vec<String>,
    },
}
