//! Refuses a restore that would write over data the caller did not expect.

use super::super::RestoreError;
use crate::infra::docker::{PostgresTarget, table_counts};

/// Schema inspected when deciding whether a target is empty.
const INSPECTED_SCHEMA: &str = "public";

impl From<bool> for RestoreAuthorisation {
    /// Maps the `--force` flag onto the authorisation it grants.
    fn from(force: bool) -> Self {
        match force {
            true => Self::Forced,
            false => Self::RequireEmpty,
        }
    }
}

/// How the caller authorised this restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreAuthorisation {
    /// No override: a target holding data is refused.
    RequireEmpty,
    /// `--force` was passed, so a populated target is accepted.
    Forced,
}

/// Fails unless `target` may be written to.
///
/// A target holding tables is refused by default. This is the only command that
/// writes into a database the user already runs, so the default answer is no.
pub async fn ensure_writable(
    target: &PostgresTarget,
    authorisation: &RestoreAuthorisation,
) -> Result<(), RestoreError> {
    if *authorisation == RestoreAuthorisation::Forced {
        return Ok(());
    }

    let populated = populated_tables(target).await?;
    if populated == 0 {
        return Ok(());
    }

    Err(RestoreError::TargetNotEmpty {
        container: target.container.clone(),
        database: target.database.clone(),
        tables: populated,
    })
}

/// Counts tables in the inspected schema that hold at least one row.
pub async fn populated_tables(target: &PostgresTarget) -> Result<usize, RestoreError> {
    let counts = table_counts(target, INSPECTED_SCHEMA).await?;
    Ok(counts.values().filter(|rows| **rows > 0).count())
}

#[cfg(test)]
#[path = "guard_test.rs"]
mod guard_test;
