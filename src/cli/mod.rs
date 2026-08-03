//! The command-line surface: parse, dispatch, hand a report back.
//!
//! Nothing here writes to stdout. Commands return a report and the binary
//! entrypoint prints it, so stdout has exactly one owner.

mod args;
mod commands;
mod dto;
mod errors;
mod reporter;

use clap::Parser;

pub use args::Cli;
pub use dto::CommandReport;
pub use errors::{CliError, ExitStatus};

use crate::infra::logging::{LogDetail, install};

/// Parses arguments and runs the requested command.
pub async fn run() -> Result<CommandReport, CliError> {
    let cli = Cli::parse();
    install(log_detail(&cli));

    commands::dispatch(cli).await
}

/// How much internal detail this invocation asked for.
fn log_detail(cli: &Cli) -> LogDetail {
    match cli.verbose {
        true => LogDetail::Verbose,
        false => LogDetail::Normal,
    }
}
