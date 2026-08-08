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
fn retaining_none_drops_every_archive_for_the_label() {
    let stored = keys(&[
        "app-backup-20260801-0900.dump",
        "app-backup-20260802-0900.dump",
    ]);

    assert_eq!(archives_to_drop(&stored, "app", 0).len(), 2);
}
