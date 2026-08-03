//! Row-count comparison between a source database and a restored copy.
//!
//! This is the pass/fail signal for a restore. `pg_restore`'s exit code is not:
//! restoring into a managed Postgres image reports dozens of errors for system
//! objects the image already owns, while every application row lands intact.

use std::collections::BTreeMap;

/// Row counts for one schema, keyed by table name.
pub type TableCounts = BTreeMap<String, i64>;

/// How one table's restored count relates to the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountVerdict {
    /// Identical row counts.
    Identical,
    /// The restored copy has fewer rows. Expected when the source kept taking
    /// writes after the dump was taken; the gap is drift, not loss.
    Behind { source: i64, restored: i64 },
    /// The restored copy has MORE rows, or the table is missing on one side.
    /// Neither can be explained by drift, so both fail a verification.
    Impossible { source: i64, restored: i64 },
}

impl CountVerdict {
    /// True when this verdict alone should fail the verification.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Impossible { .. })
    }
}

/// One table's comparison result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableComparison {
    /// The table this verdict is about.
    pub table: String,
    /// How the restored count relates to the source count.
    pub verdict: CountVerdict,
}

/// Compares every table present on either side.
///
/// A table missing from one side counts as `Impossible` rather than being
/// skipped, so a dump that silently dropped a table cannot pass.
pub fn compare_counts(source: &TableCounts, restored: &TableCounts) -> Vec<TableComparison> {
    let mut tables: Vec<&String> = source.keys().chain(restored.keys()).collect();
    tables.sort_unstable();
    tables.dedup();

    tables
        .into_iter()
        .map(|table| TableComparison {
            table: table.clone(),
            verdict: verdict_for(source.get(table), restored.get(table)),
        })
        .collect()
}

/// Total rows that the restored copy is behind across all drifting tables.
pub fn rows_behind(comparisons: &[TableComparison]) -> i64 {
    comparisons
        .iter()
        .filter_map(|comparison| match comparison.verdict {
            CountVerdict::Behind { source, restored } => Some(source - restored),
            CountVerdict::Identical | CountVerdict::Impossible { .. } => None,
        })
        .sum()
}

/// Decides one table's verdict from the two optional counts.
fn verdict_for(source: Option<&i64>, restored: Option<&i64>) -> CountVerdict {
    match (source, restored) {
        (Some(&source), Some(&restored)) if source == restored => CountVerdict::Identical,
        (Some(&source), Some(&restored)) if source > restored => {
            CountVerdict::Behind { source, restored }
        }
        (Some(&source), Some(&restored)) => CountVerdict::Impossible { source, restored },
        (Some(&source), None) => CountVerdict::Impossible {
            source,
            restored: 0,
        },
        (None, Some(&restored)) => CountVerdict::Impossible {
            source: 0,
            restored,
        },
        (None, None) => CountVerdict::Identical,
    }
}

#[cfg(test)]
#[path = "table_count_test.rs"]
mod table_count_test;
