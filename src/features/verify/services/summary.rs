//! How a verification describes itself.
//!
//! Leads with the verdict, then the evidence behind it. Drift is named as
//! drift: a source that kept taking writes after the dump is the normal case,
//! and reading that as damage is the mistake this output exists to prevent.

use thousands::Separable;

use super::super::{ChecksumOutcome, VerifyOutcome};
use crate::domain::CountVerdict;

/// Renders the summary lines that follow a verification's step output.
pub fn summarise(outcome: &VerifyOutcome) -> Vec<String> {
    let mut lines = vec![headline(outcome)];
    lines.push(format!(
        "      {} tables compared",
        outcome.comparisons.len()
    ));

    if outcome.rows_behind > 0 {
        lines.push(format!(
            "      {} rows behind the source, which kept writing after the dump",
            outcome.rows_behind.separate_with_commas()
        ));
    }

    lines.push(format!(
        "      {} pg_restore errors, which do not decide the result",
        outcome.restore_errors
    ));
    lines.push(format!(
        "      checksum: {}",
        checksum_line(&outcome.checksum)
    ));

    for failure in outcome.failures() {
        lines.push(format!(
            "      MISMATCH {}: {}",
            failure.table,
            verdict_line(&failure.verdict)
        ));
    }

    lines
}

/// The verdict line, which is the only line a user has to read.
fn headline(outcome: &VerifyOutcome) -> String {
    if outcome.passed() {
        format!(
            "PASS  {} restored into a scratch database and matched the source",
            outcome.archive
        )
    } else {
        format!("FAIL  {} did not match the source", outcome.archive)
    }
}

/// One-line description of a checksum outcome.
fn checksum_line(outcome: &ChecksumOutcome) -> String {
    match outcome {
        ChecksumOutcome::Matched => "matches the digest stored with the archive".to_owned(),
        ChecksumOutcome::Mismatched { expected, actual } => {
            format!("MISMATCH, stored {expected}, downloaded {actual}")
        }
        ChecksumOutcome::Absent => {
            "no digest stored with this archive, so its bytes could not be checked".to_owned()
        }
    }
}

/// One-line description of a table verdict.
fn verdict_line(verdict: &CountVerdict) -> String {
    match verdict {
        CountVerdict::Identical => "identical".to_owned(),
        CountVerdict::Behind { source, restored } => format!(
            "source {}, restored {}",
            source.separate_with_commas(),
            restored.separate_with_commas()
        ),
        CountVerdict::Impossible { source, restored } => format!(
            "source {}, restored {} -- drift cannot explain this",
            source.separate_with_commas(),
            restored.separate_with_commas()
        ),
    }
}

#[cfg(test)]
#[path = "summary_test.rs"]
mod summary_test;
