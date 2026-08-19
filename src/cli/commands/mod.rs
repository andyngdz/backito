//! One file per command: map arguments, call a feature, hand back a report.

mod backup;
mod daemon;
mod health;
mod init;
mod list;
mod restore;
mod verify;
mod walg;
mod walg_archive;
mod walg_base;
mod walg_entrypoint;

use std::path::PathBuf;
use std::sync::Arc;

use super::CliError;
use super::args::{Cli, Command, ENV_FLAG};
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
            Self::Environment => ENV_FLAG.to_owned(),
            Self::File(path) => format!("--config {}", path.display()),
        }
    }
}

/// Name `init` reports itself by when it refuses a flag.
const INIT_COMMAND: &str = "init";

/// Runs the requested command.
///
/// `init` is the one command that runs without configuration, because it is the
/// command that creates the configuration. Loading first would make the command
/// that fixes a missing config the command a missing config blocks.
pub async fn dispatch(cli: Cli) -> Result<CommandReport, CliError> {
    // Checked before the match takes ownership of the command.
    if matches!(cli.command, Command::Init { .. }) {
        cli.refuse_config_flags(INIT_COMMAND)?;
    }

    let choice = SourceChoice::from_cli(&cli);

    match cli.command {
        Command::Init { force } => init::run(force.into()),

        // `walg` loads its own settings, because its entrypoint has to echo the
        // config choice back to the command it execs.
        Command::Walg(walg) => walg::run(walg, &choice).await,

        // Listed rather than wildcarded, so a new command has to say here
        // whether it needs settings instead of silently inheriting a load.
        configured @ (Command::Backup { .. }
        | Command::Verify { .. }
        | Command::Restore { .. }
        | Command::Daemon
        | Command::List { .. }
        | Command::Health) => load(&choice)?.run(configured).await,
    }
}

impl Context {
    /// Runs the commands that need settings, once those settings are loaded.
    ///
    /// Split from `dispatch` so the load happens in one place rather than as
    /// the first line of every arm.
    async fn run(self, command: Command) -> Result<CommandReport, CliError> {
        let Self { settings, observer } = self;

        match command {
            Command::Backup { keep } => backup::run(&settings, keep.into(), observer).await,

            Command::Daemon => daemon::run(&settings, observer).await,

            Command::List { keys_only } => list::run(&settings, keys_only.into()).await,

            Command::Health => health::run(&settings, observer).await,

            Command::Verify { archive } => verify::run(&settings, archive, observer).await,

            Command::Restore {
                into_container,
                archive,
                force,
            } => restore::run(&settings, into_container, archive, force.into(), observer).await,

            // `dispatch` answers both of these before it ever gets here, so this
            // arm exists to satisfy the match and not to describe a real state.
            Command::Init { .. } | Command::Walg(_) => {
                unreachable!("dispatch handles init and walg without loading settings")
            }
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
