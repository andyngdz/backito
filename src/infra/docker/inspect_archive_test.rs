use super::count_entries;

/// Joins listing lines the way `pg_restore --list` emits them.
fn listing(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}

#[test]
fn an_empty_listing_counts_nothing() {
    assert_eq!(count_entries(""), 0);
}

#[test]
fn only_table_data_entries_are_counted() {
    let body = listing(&[
        ";     dbname: postgres",
        "215; 1259 16385 TABLE public apps postgres",
        "3456; 0 16385 TABLE DATA public apps postgres",
        "3457; 0 16386 TABLE DATA public tags postgres",
        "2890; 2606 16390 CONSTRAINT public apps apps_pkey postgres",
    ]);

    // TABLE and TABLE DATA are different entries: only the latter means rows.
    assert_eq!(count_entries(&body), 2);
}

#[test]
fn a_header_only_archive_counts_zero() {
    // This is the shape a truncated dump produces, and the reason the check
    // exists: it must not read as a valid backup.
    assert_eq!(count_entries(&listing(&[";     dbname: postgres"])), 0);
}
