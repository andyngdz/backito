use super::run;
use crate::domain::Interval;
use crate::infra::config::{
    ContainerSource, DatabaseSettings, ScheduleSettings, Settings, StorageCredentials,
    StorageSettings, WalgCredentials, WalgMode, WalgSettings,
};
use std::path::Path;

fn settings(walg: WalgMode) -> Settings {
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
        walg,
    }
}

fn enabled() -> WalgMode {
    WalgMode::Enabled(Box::new(WalgSettings {
        s3_prefix: "s3://app-walg/".to_owned(),
        endpoint: "https://account.r2.cloudflarestorage.com".to_owned(),
        region: "auto".to_owned(),
        data_dir: "/var/lib/postgresql/data".to_owned(),
        base_interval: Interval::from_secs(24 * 60 * 60),
        retain_full: 3,
        binary: "wal-g".to_owned(),
        credentials: WalgCredentials {
            access_key_id: "walg-key".to_owned(),
            secret_access_key: "walg-secret".to_owned(),
        },
    }))
}

#[test]
fn a_configured_cluster_gets_archiving_turned_on() {
    let fragment = tempfile::NamedTempFile::new().expect("temp fragment");

    // The handover fails because the program does not exist, which is what lets
    // this test read the file: a successful exec would replace the test process.
    let _ = run(
        &settings(enabled()),
        fragment.path(),
        Path::new("/etc/backito/backito.toml"),
        "backito-no-such-entrypoint",
        &["postgres".to_owned()],
    );

    let written = std::fs::read_to_string(fragment.path()).expect("read fragment");
    assert!(written.contains("archive_mode = on"), "got: {written}");
    assert!(written.contains("walg archive %p"), "got: {written}");
    // Postgres runs archive_command from its own directory, so a relative
    // config path would not be found.
    assert!(
        written.contains("/etc/backito/backito.toml"),
        "the config path must be absolute in the command, got: {written}"
    );
}

#[test]
fn a_cluster_without_wal_storage_is_left_archiving_nothing() {
    let fragment = tempfile::NamedTempFile::new().expect("temp fragment");

    let _ = run(
        &settings(WalgMode::Disabled),
        fragment.path(),
        Path::new("backito.toml"),
        "backito-no-such-entrypoint",
        &["postgres".to_owned()],
    );

    let written = std::fs::read_to_string(fragment.path()).expect("read fragment");
    assert!(!written.contains("archive_mode = on"), "got: {written}");
}
