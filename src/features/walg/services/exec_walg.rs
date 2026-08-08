//! Handing this process over to `wal-g`.

use std::os::unix::process::CommandExt;
use std::process::Command;

use super::super::WalgError;
use super::environment::walg_environment;
use crate::infra::config::WalgSettings;

/// Replaces this process with `wal-g <args>`.
///
/// `exec` rather than spawn-and-wait, so the process count stays at one. This
/// matters most for `archive`, which Postgres runs once per WAL segment: an
/// extra process per segment is a cost paid forever, and there is nothing for
/// backito to do after handing over anyway.
///
/// Returns only on failure. A successful call never comes back, because this
/// process no longer exists.
pub fn exec_walg(settings: &WalgSettings, args: &[&str]) -> WalgError {
    let mut command = Command::new(&settings.binary);
    command.args(args);
    for (name, value) in walg_environment(settings) {
        command.env(name, value);
    }

    let failure = command.exec();

    WalgError::Exec {
        binary: settings.binary.clone(),
        source: failure,
    }
}

/// Replaces this process with `program <args>`, carrying nothing of wal-g's.
///
/// Used by `entrypoint`, which hands over to the image's own entrypoint once it
/// has written the Postgres configuration.
pub fn exec_program(program: &str, args: &[String]) -> WalgError {
    let failure = Command::new(program).args(args).exec();

    WalgError::Exec {
        binary: program.to_owned(),
        source: failure,
    }
}

#[cfg(test)]
#[path = "exec_walg_test.rs"]
mod exec_walg_test;
