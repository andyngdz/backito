//! Binary entrypoint for `backito`. Wiring only: runs the CLI, prints the
//! report on stdout and any failure on stderr, and maps the outcome to an exit
//! code. Progress output is written to stderr by the CLI, so stdout carries
//! only the result and stays safe to capture.

mod cli;
mod domain;
mod features;
mod infra;

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match cli::run().await {
        Ok(report) => {
            for line in report.lines {
                println!("{line}");
            }
            ExitCode::from(report.status.code())
        }
        Err(cli_err) => {
            eprintln!("error: {cli_err}");
            if let Some(hint) = cli_err.hint() {
                eprintln!("hint:  {hint}");
            }
            ExitCode::from(cli::ExitStatus::Failure.code())
        }
    }
}
