//! Hashes the archive and sends it, with its checksum sidecar, to the bucket.

use std::path::Path;
use std::sync::Arc;

use super::super::BackupError;
use super::checksum::digest_file;
use crate::domain::{ArchiveDigest, ArchiveName};
use crate::features::progress::{ProgressObserver, Step, human_bytes};
use crate::infra::object_store::ObjectStore;

/// What reached the bucket.
#[derive(Debug)]
pub struct PublishedArchive {
    /// Digest written alongside the archive.
    pub digest: ArchiveDigest,
    /// Size the store reports for the uploaded object.
    pub stored_bytes: u64,
}

/// Hashes `archive_path`, uploads it as `archive`, then writes the sidecar.
///
/// The sidecar goes second on purpose: a checksum present in the bucket implies
/// the archive it names is already there.
pub async fn publish_archive(
    store: &ObjectStore,
    archive: &ArchiveName,
    archive_path: &Path,
    local_bytes: u64,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<PublishedArchive, BackupError> {
    observer.step_started(Step::Checksum);
    observer.transfer_started(Some(local_bytes));
    let digest = digest_file(archive_path, observer).await?;
    observer.transfer_finished();
    observer.step_finished(Step::Checksum, digest.as_str());

    observer.step_started(Step::Upload);
    observer.transfer_started(Some(local_bytes));
    let metered = observer.metered_reader();
    store
        .upload_file(archive.as_str(), archive_path, move |file| metered(file))
        .await?;
    store
        .upload_bytes(
            &archive.checksum_key(),
            digest.to_sidecar(archive).as_bytes(),
        )
        .await?;
    observer.transfer_finished();

    let stored_bytes = store.object_size(archive.as_str()).await?;
    observer.step_finished(Step::Upload, &human_bytes(stored_bytes));

    Ok(PublishedArchive {
        digest,
        stored_bytes,
    })
}

#[cfg(test)]
#[path = "publish_archive_test.rs"]
mod publish_archive_test;
