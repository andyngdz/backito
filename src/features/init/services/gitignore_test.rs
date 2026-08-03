use super::{covers, ensure_ignored};
use crate::features::init::IgnoreOutcome;
use indoc::indoc;
use tempfile::TempDir;

const ENTRY: &str = "backito.toml";

/// An ignore file a real project would already have.
fn existing_rules() -> &'static str {
    indoc! {"
        /target
        node_modules/
    "}
}

fn ignore_body(directory: &TempDir) -> String {
    std::fs::read_to_string(directory.path().join(".gitignore")).expect("read .gitignore")
}

#[test]
fn a_missing_ignore_file_is_created_with_the_entry() {
    let directory = TempDir::new().expect("temp dir");

    let outcome = ensure_ignored(directory.path(), ENTRY).expect("ensure");

    assert!(matches!(outcome, IgnoreOutcome::Created { .. }));
    assert!(ignore_body(&directory).contains(ENTRY));
}

#[test]
fn an_existing_ignore_file_keeps_what_it_had() {
    let directory = TempDir::new().expect("temp dir");
    std::fs::write(directory.path().join(".gitignore"), existing_rules()).expect("seed .gitignore");

    let outcome = ensure_ignored(directory.path(), ENTRY).expect("ensure");

    assert!(matches!(outcome, IgnoreOutcome::Appended { .. }));
    let body = ignore_body(&directory);
    assert!(body.contains("/target"), "existing rules must survive");
    assert!(body.contains("node_modules/"));
    assert!(body.contains(ENTRY));
}

#[test]
fn a_file_without_a_trailing_newline_does_not_get_a_glued_entry() {
    let directory = TempDir::new().expect("temp dir");
    std::fs::write(directory.path().join(".gitignore"), "/target").expect("seed");

    ensure_ignored(directory.path(), ENTRY).expect("ensure");

    let body = ignore_body(&directory);
    assert!(
        !body.contains("/targetbackito"),
        "entry was glued onto the previous line: {body:?}"
    );
    assert!(covers(&body, ENTRY));
}

#[test]
fn running_twice_does_not_duplicate_the_entry() {
    let directory = TempDir::new().expect("temp dir");

    ensure_ignored(directory.path(), ENTRY).expect("first");
    let second = ensure_ignored(directory.path(), ENTRY).expect("second");

    assert!(matches!(second, IgnoreOutcome::AlreadyIgnored { .. }));
    assert_eq!(ignore_body(&directory).matches(ENTRY).count(), 1);
}

#[test]
fn an_anchored_entry_already_counts_as_ignored() {
    // `/backito.toml` ignores the same file; adding a second rule would be noise.
    assert!(covers("/backito.toml\n", ENTRY));
}

#[test]
fn a_longer_name_sharing_the_prefix_does_not_count() {
    // Substring matching would see this as covered and skip a real entry.
    assert!(!covers("backito.toml.bak\n", ENTRY));
}

#[test]
fn a_commented_out_entry_does_not_count() {
    assert!(!covers("# backito.toml\n", ENTRY));
}
