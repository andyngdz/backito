use super::run;
use crate::cli::CliError;
use crate::features::list::Detail;
use crate::features::verify::VerifyError;
use crate::infra::config::{
    ContainerSource, DatabaseSettings, ScheduleSettings, Settings, StorageCredentials,
    StorageSettings, WalgMode,
};

fn unreachable_settings() -> Settings {
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
            endpoint: "http://127.0.0.1:1".to_owned(),
            bucket: "unreachable".to_owned(),
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
async fn an_unreachable_bucket_fails_rather_than_reporting_an_empty_bucket() {
    // "no archives stored yet" and "the bucket could not be read" look the same
    // on screen and mean opposite things. Only the second may fail, and it must.
    let failure = run(&unreachable_settings(), Detail::Full)
        .await
        .expect_err("a bucket that cannot be listed says nothing about archives");

    assert!(
        matches!(failure, CliError::Verify(VerifyError::Storage(_))),
        "expected a storage failure, got {failure:?}"
    );
}
