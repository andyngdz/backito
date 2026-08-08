use super::run;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::features::restore::RestoreAuthorisation;
use crate::infra::config::{DatabaseSettings, Settings, StorageCredentials, StorageSettings};
use std::sync::Arc;

fn settings() -> Settings {
    Settings {
        database: DatabaseSettings {
            label: "backito-test".to_owned(),
            container: "backito-container-that-does-not-exist".to_owned(),
            name: "postgres".to_owned(),
            user: "postgres".to_owned(),
            image: "postgres:17".to_owned(),
            restore_jobs: 4,
        },
        storage: StorageSettings {
            endpoint: "http://127.0.0.1:1".to_owned(),
            bucket: "backito-test".to_owned(),
            region: "auto".to_owned(),
        },
        credentials: StorageCredentials {
            access_key_id: "test-access-key".to_owned(),
            secret_access_key: "test-secret-key".to_owned(),
        },
    }
}

#[tokio::test]
async fn a_missing_target_container_stops_before_anything_is_downloaded() {
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = run(
        &settings(),
        None,
        None,
        RestoreAuthorisation::Forced,
        observer,
    )
    .await
    .expect_err("a missing container must fail");

    // The container check runs first on purpose: fetching a gigabyte before
    // discovering there is nowhere to put it wastes the user's time.
    assert!(failure.to_string().contains("not running"));
}
