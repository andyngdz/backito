use super::{RestoreRequest, run_restore};
use crate::domain::ArchiveChoice;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::features::restore::RestoreAuthorisation;
use crate::infra::config::{StorageCredentials, StorageSettings};
use crate::infra::docker::PostgresTarget;
use crate::infra::object_store::ObjectStore;
use std::sync::Arc;
use tempfile::TempDir;

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

fn missing_target() -> PostgresTarget {
    PostgresTarget {
        container: "backito-container-that-does-not-exist".to_owned(),
        database: "postgres".to_owned(),
        user: "postgres".to_owned(),
    }
}

#[tokio::test]
async fn the_target_is_checked_before_anything_is_downloaded() {
    let workspace = TempDir::new().expect("temp dir");
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = run_restore(
        &unreachable_store(),
        "backito-test",
        &missing_target(),
        workspace.path(),
        RestoreRequest {
            archive: ArchiveChoice::Newest,
            authorisation: RestoreAuthorisation::Forced,
            jobs: 4,
        },
        observer,
    )
    .await
    .expect_err("a missing container must fail");

    // Both the store and the container are unreachable here. The message must
    // name the container: fetching a gigabyte before discovering there is
    // nowhere to put it wastes the user's time.
    assert!(
        failure.to_string().contains("not running"),
        "message was {failure}"
    );
}
