use super::summarise;
use crate::domain::{ArchiveName, CountVerdict, TableComparison};
use crate::features::verify::{ChecksumOutcome, VerifyOutcome};

fn outcome(verdicts: Vec<CountVerdict>, rows_behind: i64) -> VerifyOutcome {
    VerifyOutcome {
        archive: ArchiveName::new("app", "20260803-0942"),
        comparisons: verdicts
            .into_iter()
            .enumerate()
            .map(|(index, verdict)| TableComparison {
                table: format!("table_{index}"),
                verdict,
            })
            .collect(),
        rows_behind,
        restore_errors: 78,
        checksum: ChecksumOutcome::Matched,
    }
}

#[test]
fn the_first_line_carries_the_verdict() {
    let lines = summarise(&outcome(vec![CountVerdict::Identical], 0));

    assert!(lines[0].starts_with("PASS"));
    assert!(lines[0].contains("app-backup-20260803-0942.dump"));
}

#[test]
fn restore_errors_are_shown_but_disclaimed() {
    let lines = summarise(&outcome(vec![CountVerdict::Identical], 0)).join("\n");

    assert!(lines.contains("78 pg_restore errors"));
    assert!(lines.contains("do not decide the result"));
}

#[test]
fn drift_is_named_as_drift_not_as_loss() {
    let lines = summarise(&outcome(
        vec![CountVerdict::Behind {
            source: 3_032_936,
            restored: 3_032_480,
        }],
        591,
    ))
    .join("\n");

    assert!(lines.contains("591 rows behind"));
    assert!(lines.contains("kept writing after the dump"));
}

#[test]
fn a_zero_drift_run_does_not_mention_drift_at_all() {
    let lines = summarise(&outcome(vec![CountVerdict::Identical], 0)).join("\n");

    assert!(!lines.contains("rows behind"));
}

#[test]
fn an_impossible_count_is_listed_with_both_sides() {
    let lines = summarise(&outcome(
        vec![CountVerdict::Impossible {
            source: 10,
            restored: 11,
        }],
        0,
    ))
    .join("\n");

    assert!(lines.starts_with("FAIL"));
    assert!(lines.contains("MISMATCH table_0"));
    assert!(lines.contains("drift cannot explain this"));
}
