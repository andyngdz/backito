use super::{CountVerdict, TableCounts, compare_counts, rows_behind};

fn counts(pairs: &[(&str, i64)]) -> TableCounts {
    pairs
        .iter()
        .map(|(table, rows)| ((*table).to_owned(), *rows))
        .collect()
}

#[test]
fn equal_counts_are_identical() {
    let comparisons = compare_counts(&counts(&[("apps", 241213)]), &counts(&[("apps", 241213)]));

    assert_eq!(comparisons[0].verdict, CountVerdict::Identical);
    assert!(!comparisons[0].verdict.is_failure());
}

#[test]
fn a_restored_copy_behind_the_source_is_drift_not_failure() {
    // The source kept scraping after the dump was taken.
    let comparisons = compare_counts(
        &counts(&[("app_player_counts", 3032936)]),
        &counts(&[("app_player_counts", 3032480)]),
    );

    assert_eq!(
        comparisons[0].verdict,
        CountVerdict::Behind {
            source: 3032936,
            restored: 3032480,
        }
    );
    assert!(!comparisons[0].verdict.is_failure());
    assert_eq!(rows_behind(&comparisons), 456);
}

#[test]
fn a_restored_copy_ahead_of_the_source_fails() {
    // Nothing writes to the drill container, so more rows than the source means
    // the comparison itself is wrong -- wrong archive, wrong database, or a
    // restore layered onto existing data.
    let comparisons = compare_counts(&counts(&[("apps", 10)]), &counts(&[("apps", 11)]));

    assert!(comparisons[0].verdict.is_failure());
}

#[test]
fn a_table_missing_from_the_restore_fails() {
    let comparisons = compare_counts(&counts(&[("apps", 10)]), &counts(&[]));

    assert!(comparisons[0].verdict.is_failure());
}

#[test]
fn a_table_only_in_the_restore_fails() {
    let comparisons = compare_counts(&counts(&[]), &counts(&[("apps", 10)]));

    assert!(comparisons[0].verdict.is_failure());
}

#[test]
fn every_table_on_either_side_gets_a_verdict() {
    let comparisons = compare_counts(
        &counts(&[("apps", 1), ("tags", 2)]),
        &counts(&[("apps", 1), ("widgets", 3)]),
    );

    let tables: Vec<&str> = comparisons
        .iter()
        .map(|comparison| comparison.table.as_str())
        .collect();
    assert_eq!(tables, vec!["apps", "tags", "widgets"]);
}
