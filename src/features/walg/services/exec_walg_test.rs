use super::{exec_program, exec_walg};
use crate::domain::Interval;
use crate::infra::config::{WalgCredentials, WalgSettings};

fn settings_with_binary(binary: &str) -> WalgSettings {
    WalgSettings {
        s3_prefix: "s3://app-walg/".to_owned(),
        endpoint: "https://account.r2.cloudflarestorage.com".to_owned(),
        region: "auto".to_owned(),
        data_dir: "/var/lib/postgresql/data".to_owned(),
        base_interval: Interval::from_secs(24 * 60 * 60),
        retain_full: 3,
        binary: binary.to_owned(),
    }
}

fn credentials() -> WalgCredentials {
    WalgCredentials {
        access_key_id: "walg-key".to_owned(),
        secret_access_key: "walg-secret".to_owned(),
    }
}

#[test]
fn a_missing_wal_g_binary_names_what_could_not_be_started() {
    // exec only returns on failure, so reaching the assertion at all is the
    // test: a successful exec would have replaced this process.
    let failure = exec_walg(
        &settings_with_binary("backito-no-such-wal-g-binary"),
        &credentials(),
        &["backup-list"],
    );

    let message = failure.to_string();
    assert!(
        message.contains("backito-no-such-wal-g-binary"),
        "the failure should name the binary, got: {message}"
    );
}

#[test]
fn a_missing_entrypoint_program_names_what_could_not_be_started() {
    let failure = exec_program("backito-no-such-entrypoint", &["postgres".to_owned()]);

    let message = failure.to_string();
    assert!(
        message.contains("backito-no-such-entrypoint"),
        "the failure should name the program, got: {message}"
    );
}
