//! Builds the per-table row-count query and reads its output.
//!
//! Pure string work, kept apart from process handling so it can be tested
//! without a database.

use crate::domain::TableCounts;

/// SQL listing every base table in one schema.
pub const LIST_TABLES_SQL: &str = "select c.relname from pg_class c join pg_namespace n on n.oid = c.relnamespace where n.nspname = '{schema}' and c.relkind = 'r' order by c.relname";

/// Renders [`LIST_TABLES_SQL`] for `schema`.
pub fn list_tables_sql(schema: &str) -> String {
    LIST_TABLES_SQL.replace("{schema}", &escape_literal(schema))
}

/// Builds one statement counting every table in `tables`, or `None` when the
/// schema holds no tables.
///
/// One statement means one round trip, which matters when a schema has 44
/// tables and the comparison runs twice.
pub fn counts_sql(schema: &str, tables: &[String]) -> Option<String> {
    if tables.is_empty() {
        return None;
    }

    let selects: Vec<String> = tables
        .iter()
        .map(|table| {
            format!(
                "select '{}' as t, count(*) as n from \"{}\".\"{}\"",
                escape_literal(table),
                escape_identifier(schema),
                escape_identifier(table)
            )
        })
        .collect();

    Some(selects.join(" union all "))
}

/// Parses `psql -tAF\t` output into table counts, ignoring blank lines.
pub fn parse_counts(stdout: &str) -> TableCounts {
    stdout
        .lines()
        .filter_map(|line| {
            let (table, rows) = line.split_once('\t')?;
            let rows = rows.trim().parse::<i64>().ok()?;
            Some((table.trim().to_owned(), rows))
        })
        .collect()
}

/// Parses one identifier per line, ignoring blanks.
pub fn parse_identifiers(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Escapes a value going inside single quotes.
fn escape_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Escapes an identifier going inside double quotes.
fn escape_identifier(value: &str) -> String {
    value.replace('"', "\"\"")
}

#[cfg(test)]
#[path = "count_query_test.rs"]
mod count_query_test;
