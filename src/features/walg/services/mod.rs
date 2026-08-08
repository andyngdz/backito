//! One file per thing the walg commands do.

pub mod backup_list;
mod environment;
mod exec_walg;
mod postgres_fragment;
mod run_base;

pub use exec_walg::{exec_program, exec_walg};
pub use postgres_fragment::{archiving_fragment, disabled_fragment};
pub use run_base::run_base_loop;
