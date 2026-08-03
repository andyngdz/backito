use super::{CliError, ExitStatus};
use crate::features::restore::RestoreError;
use crate::features::verify::VerifyError;
use crate::infra::config::ConfigError;
use crate::infra::object_store::ObjectStoreError;

/// Builds a store failure that answered with `status`.
fn store_status(status: u16) -> ObjectStoreError {
    ObjectStoreError::Status {
        operation: "download".to_owned(),
        key: "app-backup-20260803-0942.dump".to_owned(),
        status,
    }
}

#[test]
fn exit_codes_are_distinct_and_stable() {
    // A cron job branches on these, so they are a contract.
    assert_eq!(ExitStatus::Success.code(), 0);
    assert_eq!(ExitStatus::Failure.code(), 1);
    assert_eq!(ExitStatus::Mismatch.code(), 2);
}

#[test]
fn a_missing_config_tells_the_user_how_to_supply_one() {
    let failure = CliError::Config(ConfigError::ReadFile {
        path: "backito.toml".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
    });

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("backito.toml"));
    assert!(hint.contains("--config"));
}

#[test]
fn a_missing_credential_names_both_variables() {
    let failure = CliError::Config(ConfigError::MissingCredential {
        variable: "BACKITO_ACCESS_KEY_ID".to_owned(),
    });

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("BACKITO_ACCESS_KEY_ID"));
    assert!(hint.contains("BACKITO_SECRET_ACCESS_KEY"));
}

#[test]
fn a_missing_archive_points_at_the_bucket_contents_not_the_credential() {
    // The bucket and credential are fine here -- only the key is absent, so the
    // hint must not send the user auditing access.
    let failure = CliError::Verify(VerifyError::Storage(store_status(404)));

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("list the bucket"));
    assert!(hint.contains("--archive"));
    assert!(!hint.contains("credential"));
}

#[test]
fn a_refused_request_still_points_at_the_bucket_and_credential() {
    let failure = CliError::Verify(VerifyError::Storage(store_status(403)));

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("credential"));
    assert!(hint.contains("endpoint"));
}

#[test]
fn a_restore_from_a_missing_archive_carries_the_same_hint() {
    let failure = CliError::Restore(RestoreError::Storage(store_status(404)));

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("list the bucket"));
}

#[test]
fn a_failure_with_no_useful_next_step_offers_no_hint() {
    let failure = CliError::WorkingDirectory {
        source: std::io::Error::other("disk full"),
    };

    // Better to say nothing than to invent advice that does not apply.
    assert!(failure.hint().is_none());
}
