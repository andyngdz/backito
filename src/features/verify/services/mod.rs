//! The work a verification performs.

mod fetch_archive;
mod run_verify;
mod scratch;
mod summary;

pub use run_verify::run_verify;
pub use summary::summarise;
