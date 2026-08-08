//! `backito walg`: physical backups, driven through the `wal-g` binary.
//!
//! backito does not reimplement WAL archiving. It owns the configuration, the
//! cadence and the reporting, and hands the actual work to wal-g.

mod errors;
mod services;

pub use errors::WalgError;
pub use services::{archiving_fragment, disabled_fragment, exec_program, exec_walg, run_base_loop};
