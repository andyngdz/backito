use super::run_walg;
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

#[tokio::test]
async fn a_missing_wal_g_binary_names_what_could_not_be_started() {
    let failure = run_walg(
        &settings_with_binary("backito-no-such-wal-g-binary"),
        &credentials(),
        &["backup-list"],
    )
    .await
    .expect_err("a binary that is not installed cannot run");

    let message = failure.to_string();
    assert!(
        message.contains("backito-no-such-wal-g-binary"),
        "the failure should name the binary, got: {message}"
    );
}

#[tokio::test]
async fn a_non_zero_exit_names_the_operation_that_failed() {
    // `false` stands in for wal-g: it exits 1 and says nothing, which is the
    // shape of a wal-g run that could not reach its bucket.
    let failure = run_walg(
        &settings_with_binary("false"),
        &credentials(),
        &["backup-push"],
    )
    .await
    .expect_err("a non-zero exit is a failure");

    let message = failure.to_string();
    assert!(
        message.contains("backup-push"),
        "the failure should name the operation, got: {message}"
    );
}

#[tokio::test]
async fn both_streams_reach_the_parser() {
    // wal-g moved its INFO lines between stdout and stderr across versions, so
    // reading only one of them makes the listing look empty on the other.
    let listing = run_walg(
        &settings_with_binary("sh"),
        &credentials(),
        &["-c", "echo out-line; echo err-line >&2"],
    )
    .await
    .expect("a command that exits zero");

    assert!(listing.contains("out-line"), "got: {listing}");
    assert!(listing.contains("err-line"), "got: {listing}");
}
