//! One file per command: map arguments, call a feature, hand back a report.

mod backup;
mod daemon;
mod health;
mod init;
mod restore;
mod verify;
mod walg;
mod walg_archive;
mod walg_base;
mod walg_entrypoint;

use std::path::Path;
use std::sync::Arc;

use super::CliError;
use super::args::{Cli, Command};
use super::dto::CommandReport;
use super::reporter::TerminalReporter;
use crate::features::progress::ProgressObserver;
use crate::infra::config::Settings;

/// What every command except `init` needs before it can start.
struct Context {
    settings: Settings,
    observer: Arc<dyn ProgressObserver>,
}

/// Runs the requested command.
///
/// `init` is the one command that runs without configuration, because it is the
/// command that creates the configuration. Loading first would make the command
/// that fixes a missing config the command a missing config blocks.
pub async fn dispatch(cli: Cli) -> Result<CommandReport, CliError> {
    let Cli {
        config, command, ..
    } = cli;

    match command {
        Command::Init { force } => init::run(force.into()),

        Command::Backup { keep } => {
            let context = load(config.as_deref())?;
            backup::run(&context.settings, keep.into(), context.observer).await
        }

        Command::Daemon => {
            let context = load(config.as_deref())?;
            daemon::run(&context.settings, context.observer).await
        }

        Command::Walg(walg) => walg::run(walg, config.as_deref()).await,

        Command::Health => {
            let context = load(config.as_deref())?;
            health::run(&context.settings, context.observer).await
        }

        Command::Verify { archive } => {
            let context = load(config.as_deref())?;
            verify::run(&context.settings, archive, context.observer).await
        }

        Command::Restore {
            into_container,
            archive,
            force,
        } => {
            let context = load(config.as_deref())?;
            restore::run(
                &context.settings,
                into_container,
                archive,
                force.into(),
                context.observer,
            )
            .await
        }
    }
}

/// Reads configuration and builds the progress reporter.
fn load(config: Option<&Path>) -> Result<Context, CliError> {
    Ok(Context {
        settings: Settings::load(config)?,
        observer: Arc::new(TerminalReporter::new()),
    })
}
