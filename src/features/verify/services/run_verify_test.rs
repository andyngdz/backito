use super::count_restore_errors;

/// Joins stderr lines the way `pg_restore` emits them.
fn stderr(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}

#[test]
fn a_clean_restore_reports_no_errors() {
    assert_eq!(count_restore_errors(""), 0);
}

#[test]
fn both_error_spellings_pg_restore_uses_are_counted() {
    let body = stderr(&[
        "pg_restore: error: could not execute query",
        "ERROR:  permission denied for schema auth",
        "pg_restore: warning: errors ignored on restore: 78",
    ]);

    // The warning line is a summary, not an error; only the two real ones count.
    assert_eq!(count_restore_errors(&body), 2);
}

#[test]
fn ordinary_progress_lines_are_not_mistaken_for_errors() {
    let body = stderr(&[
        "pg_restore: connecting to database for restore",
        "pg_restore: creating TABLE \"public.apps\"",
    ]);

    assert_eq!(count_restore_errors(&body), 0);
}
