//! Loading an archive into a real database, behind a guard.

mod domain;
mod errors;
mod services;

pub use domain::RestoreOutcome;
pub use errors::RestoreError;
pub use services::{RestoreAuthorisation, run_restore};
