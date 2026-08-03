//! Setting a project up: write the starter config and keep it out of git.

mod domain;
mod errors;
mod services;

pub use domain::{IgnoreOutcome, InitOutcome};
pub use errors::{FileOperation, InitError};
pub use services::{Overwrite, run_init, summarise};
