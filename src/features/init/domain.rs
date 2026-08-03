//! What initialising a project produced.

use std::path::PathBuf;

/// Where the config landed and what happened to `.gitignore`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOutcome {
    /// Config file that was written.
    pub config_path: PathBuf,
    /// What the ignore file needed.
    pub ignore: IgnoreOutcome,
}

/// How `.gitignore` was left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IgnoreOutcome {
    /// The entry was added to an existing ignore file.
    Appended {
        /// Ignore file that was edited.
        path: PathBuf,
    },
    /// The ignore file did not exist and was created with the entry.
    Created {
        /// Ignore file that was written.
        path: PathBuf,
    },
    /// The entry was already there, so nothing changed.
    AlreadyIgnored {
        /// Ignore file that already covered it.
        path: PathBuf,
    },
}

#[cfg(test)]
#[path = "domain_test.rs"]
mod domain_test;
