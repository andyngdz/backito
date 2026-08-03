use super::{CliError, ExitStatus};
use crate::infra::config::ConfigError;

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
fn a_failure_with_no_useful_next_step_offers_no_hint() {
    let failure = CliError::WorkingDirectory {
        source: std::io::Error::other("disk full"),
    };

    // Better to say nothing than to invent advice that does not apply.
    assert!(failure.hint().is_none());
}
