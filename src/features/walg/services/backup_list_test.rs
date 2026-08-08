use super::newest_base_age;
use crate::domain::Interval;
use crate::features::daemon::ArchiveAge;
use indoc::indoc;
use jiff::Timestamp;

/// A real `wal-g backup-list` run, header and all.
fn listing() -> &'static str {
    indoc! {r#"
        backup_name                   modified             wal_file_name            storage_name
        base_000000010000000400000025 2026-08-08T18:16:20Z 000000010000000400000025 default
        base_00000001000000040000003B 2026-08-08T18:25:00Z 00000001000000040000003B default
        "#}
}

fn at(instant: &str) -> Timestamp {
    instant.parse().expect("a fixed instant")
}

#[test]
fn the_newest_row_decides_the_age() {
    // Not simply the last line: wal-g orders by name, and a row's position says
    // nothing about when it was taken.
    let age = newest_base_age(listing(), at("2026-08-08T19:25:00Z"));

    assert_eq!(age, ArchiveAge::Known(Interval::from_secs(60 * 60)));
}

#[test]
fn the_header_row_is_not_mistaken_for_a_backup() {
    let header_only = indoc! {r#"
        backup_name                   modified             wal_file_name            storage_name
        "#};

    let age = newest_base_age(header_only, at("2026-08-08T19:25:00Z"));

    assert_eq!(age, ArchiveAge::Unknown);
}

#[test]
fn an_empty_listing_means_no_base_backup_yet() {
    assert_eq!(
        newest_base_age("", at("2026-08-08T19:25:00Z")),
        ArchiveAge::Unknown
    );
}

#[test]
fn wal_g_info_lines_are_ignored() {
    // wal-g writes INFO lines onto the same stream when it is feeling chatty.
    let noisy = indoc! {r#"
        INFO: 2026/08/08 17:47:43.957144 List backups from storages: [default]
        backup_name                   modified             wal_file_name            storage_name
        base_000000010000000400000025 2026-08-08T18:16:20Z 000000010000000400000025 default
        "#};

    let age = newest_base_age(noisy, at("2026-08-08T19:16:20Z"));

    assert_eq!(age, ArchiveAge::Known(Interval::from_secs(60 * 60)));
}

#[test]
fn a_row_wal_g_writes_differently_is_skipped_rather_than_guessed_at() {
    let unexpected = "base_000000010000000400000025 not-a-timestamp x default";

    assert_eq!(
        newest_base_age(unexpected, at("2026-08-08T19:25:00Z")),
        ArchiveAge::Unknown
    );
}

#[test]
fn a_backup_stamped_in_the_future_reads_as_unknown() {
    let age = newest_base_age(listing(), at("2026-08-08T17:00:00Z"));

    assert_eq!(age, ArchiveAge::Unknown);
}
