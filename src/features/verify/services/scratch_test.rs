use super::{SCRATCH_PREFIX, leftover_exists, scratch_name};

#[test]
fn a_scratch_name_always_carries_the_prefix() {
    assert_eq!(scratch_name("app"), "backito-scratch-app");
    assert!(scratch_name("app").starts_with(SCRATCH_PREFIX));
}

#[test]
fn the_prefix_is_what_tells_scratch_apart_from_a_real_database() {
    // The guard in ScratchDatabase::start relies on this: a name that does not
    // start with the prefix is never created or removed.
    assert!(!"app-db".starts_with(SCRATCH_PREFIX));
    assert!(!"some-other-db".starts_with(SCRATCH_PREFIX));
}

#[tokio::test]
async fn no_leftover_is_reported_when_nothing_is_running() {
    let leftover = leftover_exists("backito-label-that-does-not-exist")
        .await
        .expect("probe must not fail");

    assert!(!leftover);
}
