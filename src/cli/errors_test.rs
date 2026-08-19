use super::{CliError, ExitStatus};
use crate::features::init::InitError;
use crate::features::restore::RestoreError;
use crate::features::verify::VerifyError;
use crate::infra::config::ConfigError;
use crate::infra::object_store::ObjectStoreError;

/// Builds a store failure for a key that is not in the bucket.
fn store_missing_key() -> ObjectStoreError {
    ObjectStoreError::Request {
        operation: "download".to_owned(),
        key: "app-backup-20260803-0942.dump".to_owned(),
        source: Box::new(object_store::Error::NotFound {
            path: "app-backup-20260803-0942.dump".to_owned(),
            source: "no such key".into(),
        }),
    }
}

/// Builds a store failure for a request the bucket refused.
fn store_refused() -> ObjectStoreError {
    ObjectStoreError::Request {
        operation: "download".to_owned(),
        key: "app-backup-20260803-0942.dump".to_owned(),
        source: Box::new(object_store::Error::PermissionDenied {
            path: "app-backup-20260803-0942.dump".to_owned(),
            source: "denied".into(),
        }),
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
    // Naming the command beats naming the file: there is a command that writes
    // it, so the user does not have to work out its contents themselves.
    assert!(hint.contains("backito init"));
    assert!(hint.contains("--config"));
}

#[test]
fn an_existing_config_is_not_reported_as_a_reason_to_run_init_again() {
    let failure = CliError::Init(InitError::ConfigExists {
        path: "backito.toml".into(),
    });

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("--force"));
    assert!(hint.contains("edit"));
}

#[test]
fn a_missing_variable_offers_both_ways_to_supply_it() {
    // The message already names the variable, so the hint carries the other
    // half: a file is the alternative to exporting it.
    let failure = CliError::Config(ConfigError::MissingEnvVar {
        variable: "BACKITO_ACCESS_KEY_ID".to_owned(),
    });

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("set the named variable"));
    assert!(hint.contains("--config"));
}

#[test]
fn a_missing_archive_points_at_the_bucket_contents_not_the_credential() {
    // The bucket and credential are fine here -- only the key is absent, so the
    // hint must not send the user auditing access.
    let failure = CliError::Verify(VerifyError::Storage(store_missing_key()));

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("backito list"));
    assert!(hint.contains("--archive"));
    assert!(!hint.contains("credential"));
}

#[test]
fn a_refused_request_still_points_at_the_bucket_and_credential() {
    let failure = CliError::Verify(VerifyError::Storage(store_refused()));

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("credential"));
    assert!(hint.contains("endpoint"));
}

#[test]
fn a_restore_from_a_missing_archive_carries_the_same_hint() {
    let failure = CliError::Restore(RestoreError::Storage(store_missing_key()));

    let hint = failure.hint().expect("this failure must carry a hint");
    assert!(hint.contains("backito list"));
}

#[test]
fn a_failure_with_no_useful_next_step_offers_no_hint() {
    let failure = CliError::WorkingDirectory {
        source: std::io::Error::other("disk full"),
    };

    // Better to say nothing than to invent advice that does not apply.
    assert!(failure.hint().is_none());
}
