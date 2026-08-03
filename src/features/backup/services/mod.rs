//! The work a backup performs.

mod checksum;
mod keep_archive;
mod produce_archive;
mod publish_archive;
mod run_backup;

pub use checksum::digest_file;
pub use keep_archive::keep_archive;
pub use run_backup::{run_backup, target_for};
