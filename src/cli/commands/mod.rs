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

use std::path::PathBuf;
use std::sync::Arc;

use super::CliError;
use super::args::{Cli, Command};
use super::dto::CommandReport;
use super::reporter::TerminalReporter;
use crate::features::progress::ProgressObserver;
use crate::infra::config::{
    CONFIG_FILENAME, ConfigError, EnvSecretSource, EnvSource, Settings, TomlSource,
};

/// What every command except `init` needs before it can start.
struct Context {
    settings: Settings,
    observer: Arc<dyn ProgressObserver>,
}

/// Where a command reads its non-secret settings from. Chosen once from the
/// flags so the loader and the archiving fragment cannot disagree about it.
enum SourceChoice {
    /// Read every setting from `BACKITO_*` variables.
    Environment,
    /// Read them from a TOML file.
    File(PathBuf),
}

impl SourceChoice {
    /// Reads the source `--env` and `--config` ask for. clap rejects setting
    /// both, so `env` decides only when `config` is absent.
    fn from_cli(cli: &Cli) -> Self {
        match cli.env {
            true => Self::Environment,
            false => Self::File(
                cli.config
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(CONFIG_FILENAME)),
            ),
        }
    }

    /// Loads the settings, pairing the chosen config source with the environment
    /// secret source.
    fn settings(&self) -> Result<Settings, ConfigError> {
        match self {
            Self::Environment => Settings::load(&EnvSource, &EnvSecretSource),
            Self::File(path) => Settings::load(&TomlSource::new(Some(path)), &EnvSecretSource),
        }
    }

    /// How to name this choice back to backito, for the `archive_command` the
    /// entrypoint writes. Postgres runs that from a minimal environment, so the
    /// choice has to travel as a flag rather than be re-derived.
    fn cli_flags(&self) -> String {
        match self {
            Self::Environment => "--env".to_owned(),
            Self::File(path) => format!("--config {}", path.display()),
        }
    }
}

/// Runs the requested command.
///
/// `init` is the one command that runs without configuration, because it is the
/// command that creates the configuration. Loading first would make the command
/// that fixes a missing config the command a missing config blocks.
pub async fn dispatch(cli: Cli) -> Result<CommandReport, CliError> {
    let choice = SourceChoice::from_cli(&cli);

    match cli.command {
        Command::Init { force } => init::run(force.into()),

        Command::Backup { keep } => {
            let context = load(&choice)?;
            backup::run(&context.settings, keep.into(), context.observer).await
        }

        Command::Daemon => {
            let context = load(&choice)?;
            daemon::run(&context.settings, context.observer).await
        }

        Command::Walg(walg) => walg::run(walg, &choice).await,

        Command::Health => {
            let context = load(&choice)?;
            health::run(&context.settings, context.observer).await
        }

        Command::Verify { archive } => {
            let context = load(&choice)?;
            verify::run(&context.settings, archive, context.observer).await
        }

        Command::Restore {
            into_container,
            archive,
            force,
        } => {
            let context = load(&choice)?;
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
fn load(choice: &SourceChoice) -> Result<Context, CliError> {
    Ok(Context {
        settings: choice.settings()?,
        observer: Arc::new(TerminalReporter::new()),
    })
}
