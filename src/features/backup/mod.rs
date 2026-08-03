//! Taking a backup: dump, prove the archive is whole, hash it, store it.

mod domain;
mod errors;
mod services;

pub use domain::BackupOutcome;
pub use errors::BackupError;
pub use services::{digest_file, keep_archive, run_backup, target_for};
