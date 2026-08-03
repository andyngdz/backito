//! The work initialising a project performs.

mod gitignore;
mod run_init;
mod summary;
mod write_config;

pub use run_init::run_init;
pub use summary::summarise;
pub use write_config::Overwrite;
