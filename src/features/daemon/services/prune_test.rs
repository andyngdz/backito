use super::archives_to_drop;

fn keys(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn nothing_is_dropped_while_the_bucket_is_under_the_limit() {
    let stored = keys(&[
        "app-backup-20260801-0900.dump",
        "app-backup-20260802-0900.dump",
    ]);

    assert!(archives_to_drop(&stored, "app", 7).is_empty());
}

#[test]
fn the_oldest_beyond_the_limit_are_dropped() {
    let stored = keys(&[
        "app-backup-20260803-0900.dump",
        "app-backup-20260801-0900.dump",
        "app-backup-20260802-0900.dump",
    ]);

    let dropped = archives_to_drop(&stored, "app", 2);

    assert_eq!(dropped, keys(&["app-backup-20260801-0900.dump"]));
}

#[test]
fn another_label_in_the_same_bucket_is_left_alone() {
    // This is what lets one bucket hold both the local and the deployed
    // cluster's archives: each label prunes only its own keys.
    let stored = keys(&[
        "app-backup-20260801-0900.dump",
        "app-backup-20260802-0900.dump",
        "app-prod-backup-20260801-0900.dump",
        "app-prod-backup-20260802-0900.dump",
    ]);

    let dropped = archives_to_drop(&stored, "app", 1);

    assert_eq!(dropped, keys(&["app-backup-20260801-0900.dump"]));
}

#[test]
fn objects_this_tool_did_not_write_are_never_deleted() {
    let stored = keys(&[
        "app-backup-20260801-0900.dump",
        "app-backup-20260801-0900.dump.sha256",
        "notes.txt",
        "app-backup-20260802-0900.dump",
    ]);

    let dropped = archives_to_drop(&stored, "app", 1);

    assert_eq!(dropped, keys(&["app-backup-20260801-0900.dump"]));
}

#[test]
fn the_sidecar_is_never_a_candidate_on_its_own() {
    // A `.sha256` travels with its dump and is deleted alongside it. If it were
    // ever picked as a candidate itself, `checksum_key` would build
    // `....sha256.sha256` and the real archive would outlive its checksum.
    let stored = keys(&[
        "app-backup-20260801-0900.dump.sha256",
        "app-backup-20260802-0900.dump.sha256",
    ]);

    assert!(archives_to_drop(&stored, "app", 1).is_empty());
}
