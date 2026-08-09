use super::walg_environment;
use crate::domain::Interval;
use crate::infra::config::{WalgCredentials, WalgSettings};

fn settings() -> WalgSettings {
    WalgSettings {
        s3_prefix: "s3://app-walg/".to_owned(),
        endpoint: "https://account.r2.cloudflarestorage.com".to_owned(),
        region: "auto".to_owned(),
        data_dir: "/var/lib/postgresql/data".to_owned(),
        base_interval: Interval::from_secs(24 * 60 * 60),
        retain_full: 3,
        binary: "wal-g".to_owned(),
    }
}

fn credentials() -> WalgCredentials {
    WalgCredentials {
        access_key_id: "walg-key".to_owned(),
        secret_access_key: "walg-secret".to_owned(),
    }
}

fn value_of(name: &str) -> String {
    walg_environment(&settings(), &credentials())
        .into_iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("{name} should be set for wal-g"))
}

#[test]
fn the_prefix_and_endpoint_reach_wal_g_under_the_names_it_reads() {
    assert_eq!(value_of("WALG_S3_PREFIX"), "s3://app-walg/");
    assert_eq!(
        value_of("AWS_ENDPOINT"),
        "https://account.r2.cloudflarestorage.com"
    );
    assert_eq!(value_of("AWS_REGION"), "auto");
}

#[test]
fn wal_storage_credentials_are_passed_rather_than_the_archive_stores() {
    assert_eq!(value_of("AWS_ACCESS_KEY_ID"), "walg-key");
    assert_eq!(value_of("AWS_SECRET_ACCESS_KEY"), "walg-secret");
}

#[test]
fn path_style_addressing_is_forced() {
    // R2 and most S3-compatible services address buckets by path. Without this
    // wal-g builds virtual-host URLs that resolve nowhere, and the failure
    // reads as DNS rather than as configuration.
    assert_eq!(value_of("AWS_S3_FORCE_PATH_STYLE"), "true");
}

#[test]
fn nothing_is_written_into_this_process_environment() {
    // SAFETY: reading only, and the point of the test is that the call above
    // did not set anything.
    walg_environment(&settings(), &credentials());

    assert!(
        std::env::var("WALG_S3_PREFIX").is_err(),
        "building the environment must not mutate this process"
    );
}
