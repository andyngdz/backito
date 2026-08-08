use super::run;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::infra::config::ContainerSource;
use crate::infra::config::{DatabaseSettings, Settings, StorageCredentials, StorageSettings};
use std::sync::Arc;

fn unreachable_settings() -> Settings {
    Settings {
        database: DatabaseSettings {
            label: "backito-test".to_owned(),
            container: ContainerSource::Named("backito-container-that-does-not-exist".to_owned()),
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
async fn an_unreachable_bucket_fails_rather_than_reporting_a_pass() {
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = run(&unreachable_settings(), None, observer)
        .await
        .expect_err("an unreachable bucket must fail the verification");

    // A verification that cannot fetch the archive has not verified anything;
    // it must never come back as a pass.
    assert!(!failure.to_string().is_empty());
}
