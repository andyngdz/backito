use super::{IgnoreOutcome, InitOutcome};
use std::path::PathBuf;

#[test]
fn an_outcome_names_both_files_it_touched() {
    let outcome = InitOutcome {
        config_path: PathBuf::from("/repo/backito.toml"),
        ignore: IgnoreOutcome::Appended {
            path: PathBuf::from("/repo/.gitignore"),
        },
    };

    // The user has to open the config next, so its path is the result.
    assert_eq!(outcome.config_path, PathBuf::from("/repo/backito.toml"));
    assert_eq!(
        outcome.ignore,
        IgnoreOutcome::Appended {
            path: PathBuf::from("/repo/.gitignore")
        }
    );
}

#[test]
fn the_three_ignore_outcomes_stay_distinct() {
    // Each one gets a different line on screen: created, edited, or untouched.
    let path = PathBuf::from("/repo/.gitignore");

    assert_ne!(
        IgnoreOutcome::Created { path: path.clone() },
        IgnoreOutcome::Appended { path: path.clone() }
    );
    assert_ne!(
        IgnoreOutcome::Appended { path: path.clone() },
        IgnoreOutcome::AlreadyIgnored { path }
    );
}
