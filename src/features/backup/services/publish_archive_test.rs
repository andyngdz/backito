use super::publish_archive;
use crate::domain::ArchiveName;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::infra::config::{StorageCredentials, StorageSettings};
use crate::infra::object_store::ObjectStore;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

/// A store pointed at a port nothing listens on, so every request fails at the
/// transport layer without needing a server.
fn unreachable_store() -> ObjectStore {
    ObjectStore::new(
        &StorageSettings {
            endpoint: "http://127.0.0.1:1".to_owned(),
            bucket: "backito-test".to_owned(),
            region: "auto".to_owned(),
        },
        &StorageCredentials {
            access_key_id: "test-access-key".to_owned(),
            secret_access_key: "test-secret-key".to_owned(),
        },
    )
    .expect("build store")
}

#[tokio::test]
async fn an_unreachable_store_fails_instead_of_reporting_a_backup() {
    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(b"archive bytes").expect("write");
    file.flush().expect("flush");
    let archive = ArchiveName::new("app", "20260803-0942");
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = publish_archive(&unreachable_store(), &archive, file.path(), 13, &observer)
        .await
        .expect_err("an unreachable store must fail the backup");

    // The message has to name the upload and the key, since that is all the
    // user has to work from when a bucket or credential is wrong.
    let message = failure.to_string();
    assert!(message.contains("upload"), "message was {message:?}");
    assert!(
        message.contains(archive.as_str()),
        "message was {message:?}"
    );
}
