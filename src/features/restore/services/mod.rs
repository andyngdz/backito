//! The work a restore performs.

mod guard;
mod run_restore;

pub use guard::RestoreAuthorisation;
pub use run_restore::run_restore;
