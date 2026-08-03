use super::summarise;
use crate::features::init::{IgnoreOutcome, InitOutcome};
use std::path::PathBuf;

fn outcome(ignore: IgnoreOutcome) -> InitOutcome {
    InitOutcome {
        config_path: PathBuf::from("/repo/backito.toml"),
        ignore,
    }
}

fn ignore_path() -> PathBuf {
    PathBuf::from("/repo/.gitignore")
}

#[test]
fn the_first_line_names_the_file_to_open() {
    let lines = summarise(&outcome(IgnoreOutcome::Created {
        path: ignore_path(),
    }));

    assert!(lines[0].contains("/repo/backito.toml"));
}

#[test]
fn each_ignore_outcome_reads_differently() {
    let created = summarise(&outcome(IgnoreOutcome::Created {
        path: ignore_path(),
    }))[1]
        .clone();
    let appended = summarise(&outcome(IgnoreOutcome::Appended {
        path: ignore_path(),
    }))[1]
        .clone();
    let already = summarise(&outcome(IgnoreOutcome::AlreadyIgnored {
        path: ignore_path(),
    }))[1]
        .clone();

    // "created" and "added to" are different events for the user's repo, and
    // "already ignored" must not claim a change that did not happen.
    assert!(created.contains("created"));
    assert!(appended.contains("added"));
    assert!(already.contains("already"));
}

#[test]
fn the_summary_ends_on_the_step_the_user_still_has_to_do() {
    let lines = summarise(&outcome(IgnoreOutcome::Created {
        path: ignore_path(),
    }));
    let joined = lines.join("\n");

    // The written config does not work until these are supplied, so a summary
    // that stopped at "wrote the file" would read as finished.
    assert!(joined.contains("endpoint"));
    assert!(joined.contains("bucket"));
    assert!(joined.contains("BACKITO_ACCESS_KEY_ID"));
    assert!(joined.contains("backito backup"));
}
