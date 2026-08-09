use super::run;
use crate::infra::config::{
    ContainerSource, DatabaseSettings, ScheduleSettings, Settings, StorageCredentials,
    StorageSettings, WalgMode,
};

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

#[test]
fn without_wal_storage_a_segment_is_skipped_rather_than_failed() {
    // Postgres reads a non-zero exit as "not archived, keep the segment". A
    // development container with nowhere to put WAL would then never recycle a
    // segment, and would fill its disk with WAL it can do nothing with.
    let report = run(&settings_without_walg(), "000000010000000000000001")
        .expect("a missing [walg] table must not fail archiving");

    assert_eq!(report.status, crate::cli::errors::ExitStatus::Success);
}
