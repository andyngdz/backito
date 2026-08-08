//! `backito daemon`: back up on a cadence, prune, verify now and then.
//!
//! `health` lives here too: judging whether a backup is recent enough is the
//! same archive-age question the schedule already answers, and splitting it
//! would mean two places that know how a stamp is read.

mod errors;
mod services;

pub use errors::DaemonError;
pub use services::{BackupFreshness, backup_freshness, newest_archive, run_loop};
