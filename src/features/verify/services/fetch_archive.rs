//! Fetches an archive from the bucket and checks it against its sidecar.

use std::path::Path;
use std::sync::Arc;

use super::super::{ChecksumOutcome, VerifyError};
use crate::domain::{ArchiveDigest, ArchiveName};
use crate::features::backup::digest_file;
use crate::features::progress::{ProgressObserver, Step, human_bytes};
use crate::infra::object_store::ObjectStore;

/// Downloads `archive` to `destination` and compares it to its stored digest.
pub async fn fetch_archive(
    store: &ObjectStore,
    archive: &ArchiveName,
    destination: &Path,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<ChecksumOutcome, VerifyError> {
    observer.step_started(Step::Download);
    let expected_bytes = store.object_size(archive.as_str()).await?;
    observer.transfer_started(Some(expected_bytes));
    store.download_file(archive.as_str(), destination).await?;
    observer.transfer_finished();
    observer.step_finished(Step::Download, &human_bytes(expected_bytes));

    observer.step_started(Step::Checksum);
    observer.transfer_started(Some(expected_bytes));
    let actual = digest_file(destination, observer).await?;
    observer.transfer_finished();

    let outcome = compare_to_sidecar(store, archive, &actual).await?;
    observer.step_finished(Step::Checksum, &describe(&outcome));
    Ok(outcome)
}

/// Reads the sidecar and compares it to `actual`.
async fn compare_to_sidecar(
    store: &ObjectStore,
    archive: &ArchiveName,
    actual: &ArchiveDigest,
) -> Result<ChecksumOutcome, VerifyError> {
    let sidecar = match store.download_text(&archive.checksum_key()).await {
        Ok(body) => body,
        // An archive uploaded before sidecars existed, or one whose sidecar was
        // removed: reported rather than treated as a match. Only a key the store
        // says is absent counts; a refused or unreachable request would otherwise
        // be reported as "no checksum stored" for a sidecar that is right there.
        Err(failure) if failure.is_missing_object() => return Ok(ChecksumOutcome::Absent),
        Err(other) => return Err(other.into()),
    };

    let Some(expected) = ArchiveDigest::from_sidecar(&sidecar) else {
        return Ok(ChecksumOutcome::Absent);
    };

    if &expected == actual {
        return Ok(ChecksumOutcome::Matched);
    }
    Ok(ChecksumOutcome::Mismatched {
        expected: expected.as_str().to_owned(),
        actual: actual.as_str().to_owned(),
    })
}

/// One-line summary of a checksum outcome, for the step line.
pub(super) fn describe(outcome: &ChecksumOutcome) -> String {
    match outcome {
        ChecksumOutcome::Matched => "matches stored checksum".to_owned(),
        ChecksumOutcome::Mismatched { expected, actual } => {
            format!("MISMATCH: stored {expected}, downloaded {actual}")
        }
        ChecksumOutcome::Absent => "no stored checksum to compare against".to_owned(),
    }
}

#[cfg(test)]
#[path = "fetch_archive_test.rs"]
mod fetch_archive_test;
