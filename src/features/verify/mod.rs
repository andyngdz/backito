//! Proving an archive restores: fetch it, load it into a throwaway database,
//! and compare row counts against the live source.

mod domain;
mod errors;
mod services;

pub use domain::{ChecksumOutcome, VerifyOutcome};
pub use errors::VerifyError;
pub use services::{run_verify, summarise};
