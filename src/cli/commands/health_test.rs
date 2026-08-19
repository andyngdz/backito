use super::run;
use crate::cli::CliError;
use crate::features::daemon::DaemonError;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::infra::config::WalgMode;
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
        walg: WalgMode::Disabled,
        walg_credentials: None,
    }
}

#[tokio::test]
async fn an_unreachable_bucket_fails_rather_than_reporting_healthy() {
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = run(&unreachable_settings(), observer)
        .await
        .expect_err("a bucket that cannot be read proves nothing about backups");

    // The dangerous answer here is a green healthcheck: a probe that cannot see
    // the bucket has not established that a backup exists, and saying "healthy"
    // would hide exactly the outage it is there to catch. Naming the class
    // matters too, because an empty-string assertion passes for a config parse
    // error just as happily.
    assert!(
        matches!(failure, CliError::Daemon(DaemonError::Storage(_))),
        "expected a storage failure, got {failure:?}"
    );
}
