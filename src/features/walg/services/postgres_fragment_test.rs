use super::{archiving_fragment, disabled_fragment};
use std::path::Path;

#[test]
fn archiving_names_the_config_the_command_must_read() {
    let fragment = archiving_fragment(Path::new("/etc/backito/backito.toml"));

    assert!(fragment.contains("archive_mode = on"), "got: {fragment}");
    assert!(fragment.contains("walg archive %p"), "got: {fragment}");
    // Postgres runs archive_command from its own working directory, so a
    // relative config path would resolve somewhere else or nowhere.
    assert!(
        fragment.contains("/etc/backito/backito.toml"),
        "got: {fragment}"
    );
}

#[test]
fn the_command_names_an_absolute_executable() {
    let fragment = archiving_fragment(Path::new("/etc/backito/backito.toml"));

    // Postgres runs archive_command with a minimal environment, and a bare
    // `backito` is only found when PATH happens to carry it.
    let command_line = fragment
        .lines()
        .find(|line| line.starts_with("archive_command"))
        .expect("the fragment sets archive_command");
    assert!(
        command_line.contains("/"),
        "the executable should be a path, got: {command_line}"
    );
}

#[test]
fn a_quiet_database_still_archives_within_the_timeout() {
    let fragment = archiving_fragment(Path::new("backito.toml"));

    // Without archive_timeout a database with little write traffic ships
    // nothing until a segment fills, which can be hours.
    assert!(
        fragment.contains("archive_timeout = 600"),
        "got: {fragment}"
    );
}

#[test]
fn the_disabled_fragment_turns_nothing_on() {
    let fragment = disabled_fragment();

    assert!(!fragment.contains("archive_mode = on"), "got: {fragment}");
    assert!(fragment.contains("[walg]"), "got: {fragment}");
}
