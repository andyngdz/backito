use super::{counts_sql, list_tables_sql, parse_counts, parse_identifiers};

/// Joins `psql -tAF\t` rows the way the client emits them. Built from a slice
/// rather than one literal because the separator is a tab.
fn psql_output(rows: &[&str]) -> String {
    format!("{}\n", rows.join("\n"))
}

#[test]
fn no_tables_produce_no_query() {
    assert_eq!(counts_sql("public", &[]), None);
}

#[test]
fn one_table_counts_itself_in_its_schema() {
    let sql = counts_sql("public", &["apps".to_owned()]).expect("query");

    assert_eq!(
        sql,
        "select 'apps' as t, count(*) as n from \"public\".\"apps\""
    );
}

#[test]
fn many_tables_share_one_statement() {
    let sql = counts_sql("public", &["apps".to_owned(), "tags".to_owned()]).expect("query");

    assert_eq!(sql.matches("union all").count(), 1);
}

#[test]
fn identifiers_with_quotes_cannot_escape_their_quoting() {
    let sql = counts_sql("public", &["we\"ird".to_owned()]).expect("query");

    assert!(sql.contains("\"we\"\"ird\""));
}

#[test]
fn a_schema_with_an_apostrophe_stays_inside_its_literal() {
    let sql = list_tables_sql("it's");

    assert!(sql.contains("'it''s'"));
}

#[test]
fn tab_separated_counts_parse_into_table_rows() {
    let counts = parse_counts(&psql_output(&["apps\t241213", "app_tags\t3292316"]));

    assert_eq!(counts.get("apps"), Some(&241213));
    assert_eq!(counts.get("app_tags"), Some(&3292316));
}

#[test]
fn blank_and_malformed_lines_are_skipped() {
    let counts = parse_counts(&psql_output(&["apps\t10", "", "not-a-row", "broken\tnope"]));

    assert_eq!(counts.len(), 1);
    assert_eq!(counts.get("apps"), Some(&10));
}

#[test]
fn identifier_lines_are_trimmed_and_compacted() {
    assert_eq!(
        parse_identifiers(&psql_output(&[" apps ", "", " tags"])),
        vec!["apps".to_owned(), "tags".to_owned()]
    );
}
