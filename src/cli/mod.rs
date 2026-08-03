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
use std::sync::Arc;

pub use args::Cli;
pub use dto::CommandReport;
pub use errors::{CliError, ExitStatus};

use crate::features::progress::ProgressObserver;
use crate::infra::config::Settings;
use crate::infra::logging::{LogDetail, install};
use reporter::TerminalReporter;

/// Parses arguments and runs the requested command.
pub async fn run() -> Result<CommandReport, CliError> {
    let cli = Cli::parse();
    install(log_detail(&cli));

    let observer: Arc<dyn ProgressObserver> = Arc::new(TerminalReporter::new(cli.quiet));
    let settings = Settings::load(cli.config.as_deref())?;

    commands::dispatch(cli.command, &settings, observer).await
}

/// How much internal detail this invocation asked for.
fn log_detail(cli: &Cli) -> LogDetail {
    match cli.verbose {
        true => LogDetail::Verbose,
        false => LogDetail::Normal,
    }
}
