use super::{ChecksumOutcome, VerifyOutcome};
use crate::domain::{ArchiveName, CountVerdict, TableComparison};

fn outcome(verdicts: Vec<CountVerdict>, checksum: ChecksumOutcome) -> VerifyOutcome {
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
        rows_behind: 0,
        restore_errors: 78,
        checksum,
    }
}

#[test]
fn identical_counts_with_a_matching_checksum_pass() {
    let verified = outcome(vec![CountVerdict::Identical], ChecksumOutcome::Matched);

    assert!(verified.passed());
    assert!(verified.failures().is_empty());
}

#[test]
fn seventy_eight_restore_errors_do_not_fail_a_verification() {
    // Restoring into a managed Postgres image always reports errors for system
    // objects the image already owns. Row counts are the signal, not the tally.
    let verified = outcome(vec![CountVerdict::Identical], ChecksumOutcome::Matched);

    assert_eq!(verified.restore_errors, 78);
    assert!(verified.passed());
}

#[test]
fn a_source_that_kept_writing_still_passes() {
    let verified = outcome(
        vec![CountVerdict::Behind {
            source: 3_032_936,
            restored: 3_032_480,
        }],
        ChecksumOutcome::Matched,
    );

    assert!(verified.passed());
}

#[test]
fn a_restored_table_ahead_of_the_source_fails() {
    let verified = outcome(
        vec![CountVerdict::Impossible {
            source: 10,
            restored: 11,
        }],
        ChecksumOutcome::Matched,
    );

    assert!(!verified.passed());
    assert_eq!(verified.failures().len(), 1);
}

#[test]
fn a_checksum_mismatch_fails_even_with_perfect_counts() {
    let verified = outcome(
        vec![CountVerdict::Identical],
        ChecksumOutcome::Mismatched {
            expected: "aaa".to_owned(),
            actual: "bbb".to_owned(),
        },
    );

    assert!(!verified.passed());
}

#[test]
fn an_archive_with_no_stored_checksum_is_not_considered_verified() {
    let verified = outcome(vec![CountVerdict::Identical], ChecksumOutcome::Absent);

    assert!(!verified.passed());
}
