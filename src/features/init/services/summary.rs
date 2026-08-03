//! How a finished init describes itself.

use super::super::{IgnoreOutcome, InitOutcome};

/// Renders the lines a finished init prints.
///
/// Ends on the one action left to take. `init` is only half the setup -- the
/// file it writes does not work until the endpoint and bucket are filled in --
/// so saying "created" without saying "now edit it" would read as done.
pub fn summarise(outcome: &InitOutcome) -> Vec<String> {
    vec![
        format!("wrote {}", outcome.config_path.display()),
        ignore_line(&outcome.ignore),
        String::new(),
        "Next: open that file and fill in endpoint and bucket, then export".to_owned(),
        "BACKITO_ACCESS_KEY_ID and BACKITO_SECRET_ACCESS_KEY and run: backito backup".to_owned(),
    ]
}

/// One line describing what the ignore file needed.
fn ignore_line(ignore: &IgnoreOutcome) -> String {
    match ignore {
        IgnoreOutcome::Created { path } => format!("created {} with the entry", path.display()),
        IgnoreOutcome::Appended { path } => format!("added the entry to {}", path.display()),
        IgnoreOutcome::AlreadyIgnored { path } => format!("{} already ignored it", path.display()),
    }
}

#[cfg(test)]
#[path = "summary_test.rs"]
mod summary_test;
