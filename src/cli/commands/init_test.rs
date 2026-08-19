use crate::features::init::{Overwrite, run_init, summarise};
use tempfile::TempDir;

#[test]
fn init_reports_what_it_wrote_and_says_what_to_do_next() {
    // Against a temp directory, not the working one. `run` resolves
    // `current_dir()`, which under `cargo test` is the repository root, so
    // driving it here would overwrite a real backito.toml on every test run.
    // The report shape is what this test is about, and `run_init` produces it.
    let directory = TempDir::new().expect("temp dir");

    let outcome = run_init(directory.path(), Overwrite::Refuse).expect("init");
    let joined = summarise(&outcome).join("\n");

    assert!(joined.contains("backito.toml"));
    assert!(joined.contains("backito backup"));
}

#[test]
fn a_second_init_refuses_rather_than_replacing_a_filled_in_config() {
    // The config carries an endpoint and a bucket by the time anyone runs this
    // twice, so silently rewriting it costs real configuration.
    let directory = TempDir::new().expect("temp dir");
    run_init(directory.path(), Overwrite::Refuse).expect("first init");

    let failure = run_init(directory.path(), Overwrite::Refuse).expect_err("second init");

    assert!(failure.to_string().contains("backito.toml"));
}
