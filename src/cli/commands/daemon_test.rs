use super::run;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::infra::config::{
    ContainerSource, DatabaseSettings, ScheduleSettings, Settings, StorageCredentials,
    StorageSettings,
};
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
        schedule: ScheduleSettings::default(),
    }
}

#[tokio::test]
async fn an_unreachable_bucket_stops_the_loop_before_it_starts() {
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = run(&unreachable_settings(), observer)
        .await
        .expect_err("a bucket that cannot be listed cannot be backed up to");

    // Failing at startup is the point: once the loop is running it swallows a
    // failed pass and retries, which is right for a transient outage and wrong
    // for a configuration that can never work.
    assert!(!failure.to_string().is_empty());
}
