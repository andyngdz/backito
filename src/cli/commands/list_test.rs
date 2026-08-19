use super::{Detail, render};
use crate::domain::ArchiveName;
use crate::infra::object_store::StoredArchive;

fn stored(key: &str, bytes: u64) -> StoredArchive {
    StoredArchive {
        name: ArchiveName::from_key(key),
        bytes,
    }
}

#[test]
fn the_keys_only_shape_prints_one_key_per_line_and_nothing_else() {
    // This is the shape a pipe consumes, so anything decorative in it becomes
    // an argument to whatever runs next.
    let archives = vec![
        stored("app-backup-20260801-0900.dump", 1024),
        stored("app-backup-20260802-0900.dump", 2048),
    ];

    assert_eq!(
        render(&archives, Detail::KeysOnly),
        vec![
            "app-backup-20260801-0900.dump".to_owned(),
            "app-backup-20260802-0900.dump".to_owned(),
        ]
    );
}

#[test]
fn the_full_shape_carries_the_key_its_size_and_when_it_was_taken() {
    let lines = render(
        &[stored("app-backup-20260803-0942.dump", 3 * 1024 * 1024)],
        Detail::Full,
    );

    let only = lines.first().expect("one archive, one line");
    assert!(only.contains("app-backup-20260803-0942.dump"));
    assert!(only.contains("3.00 MiB"));
    assert!(only.contains("2026-08-03 09:42 UTC"));
}

#[test]
fn keys_line_up_so_a_listing_stays_scannable() {
    let archives = vec![
        stored("app-backup-20260801-0900.dump", 1024),
        stored("a-much-longer-label-backup-20260802-0900.dump", 2048),
    ];

    let lines = render(&archives, Detail::Full);

    let size_columns: Vec<_> = lines
        .iter()
        .map(|line| line.find("KiB").expect("every row states a size"))
        .collect();
    assert_eq!(size_columns[0], size_columns[1]);
}

#[test]
fn an_empty_bucket_says_so_rather_than_printing_nothing() {
    // Someone running this straight after `init` needs to be told the bucket is
    // reachable and empty, which a blank screen does not say.
    let told = render(&[], Detail::Full);

    assert_eq!(told.len(), 1);
    assert!(told[0].contains("backito backup"));
}

#[test]
fn an_empty_bucket_prints_nothing_at_all_into_a_pipe() {
    assert!(render(&[], Detail::KeysOnly).is_empty());
}

#[test]
fn a_key_with_no_readable_stamp_is_shown_rather_than_hidden() {
    // It means something else wrote a key that passes the label filter, which
    // is worth seeing rather than quietly dropping from the listing.
    let lines = render(&[stored("app-backup-whenever.dump", 10)], Detail::Full);

    assert!(lines[0].contains("app-backup-whenever.dump"));
    assert!(lines[0].contains("unknown date"));
}

#[test]
fn the_flag_maps_onto_the_shape_it_names() {
    assert_eq!(Detail::from(true), Detail::KeysOnly);
    assert_eq!(Detail::from(false), Detail::Full);
}
