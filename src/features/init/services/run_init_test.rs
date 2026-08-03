use super::run_init;
use crate::features::init::Overwrite;
use crate::features::init::{IgnoreOutcome, InitError};
use tempfile::TempDir;

#[test]
fn a_fresh_project_gets_both_files() {
    let directory = TempDir::new().expect("temp dir");

    let outcome = run_init(directory.path(), Overwrite::Refuse).expect("init");

    assert!(outcome.config_path.exists());
    assert!(matches!(outcome.ignore, IgnoreOutcome::Created { .. }));
}

#[test]
fn a_refused_overwrite_leaves_gitignore_untouched() {
    // The config write runs first on purpose: failing there must not leave a
    // half-done setup with an ignore entry for a file that was never written.
    let directory = TempDir::new().expect("temp dir");
    std::fs::write(directory.path().join("backito.toml"), "# mine\n").expect("seed");

    let failure = run_init(directory.path(), Overwrite::Refuse).expect_err("must refuse");

    assert!(matches!(failure, InitError::ConfigExists { .. }));
    assert!(!directory.path().join(".gitignore").exists());
}

#[test]
fn a_project_whose_config_predates_this_command_still_gets_ignored() {
    let directory = TempDir::new().expect("temp dir");
    std::fs::write(directory.path().join("backito.toml"), "# mine\n").expect("seed");

    let outcome = run_init(directory.path(), Overwrite::Allow).expect("init");

    assert!(matches!(outcome.ignore, IgnoreOutcome::Created { .. }));
}
