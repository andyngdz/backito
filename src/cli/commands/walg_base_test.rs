use super::run;
use crate::features::progress::{ProgressObserver, SilentObserver};
use crate::infra::config::{
    ContainerSource, DatabaseSettings, ScheduleSettings, Settings, StorageCredentials,
    StorageSettings, WalgMode,
};
use std::sync::Arc;

fn settings_without_walg() -> Settings {
    Settings {
        database: DatabaseSettings {
            label: "app".to_owned(),
            container: ContainerSource::Named("app-db".to_owned()),
            name: "postgres".to_owned(),
            user: "postgres".to_owned(),
            image: "postgres:17".to_owned(),
            restore_jobs: 4,
        },
        storage: StorageSettings {
            endpoint: "https://account.r2.cloudflarestorage.com".to_owned(),
            bucket: "app-database-backups".to_owned(),
            region: "auto".to_owned(),
        },
        credentials: StorageCredentials {
            access_key_id: "archive-key".to_owned(),
            secret_access_key: "archive-secret".to_owned(),
        },
        schedule: ScheduleSettings::default(),
        walg: WalgMode::Disabled,
        walg_credentials: None,
    }
}

#[tokio::test]
async fn without_a_walg_table_the_loop_refuses_to_start() {
    let observer: Arc<dyn ProgressObserver> = Arc::new(SilentObserver);

    let failure = run(&settings_without_walg(), observer)
        .await
        .expect_err("there is nowhere to put a base backup");

    // The shell version slept forever here, which reads as a healthy service
    // that has quietly never backed anything up.
    assert!(
        failure.to_string().contains("[walg]"),
        "the failure should name the missing section, got: {failure}"
    );
}
