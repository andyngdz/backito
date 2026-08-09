//! The storage endpoint resolves from the config or `BACKITO_ENDPOINT`, so a
//! committed config can keep the account-carrying endpoint out of source.

use super::{ConfigError, Settings};
use crate::infra::config::{ENV_TURN, WalgMode};
use indoc::indoc;
use std::io::Write;
use tempfile::NamedTempFile;

const ACCESS_KEY_VAR: &str = "BACKITO_ACCESS_KEY_ID";
const SECRET_KEY_VAR: &str = "BACKITO_SECRET_ACCESS_KEY";
const ENDPOINT_VAR: &str = "BACKITO_ENDPOINT";
const WALG_ACCESS_KEY_VAR: &str = "BACKITO_WALG_ACCESS_KEY_ID";
const WALG_SECRET_KEY_VAR: &str = "BACKITO_WALG_SECRET_ACCESS_KEY";
const ENV_ENDPOINT: &str = "https://from-env.r2.cloudflarestorage.com";

fn write_config(body: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp config");
    file.write_all(body.as_bytes()).expect("write temp config");
    file.flush().expect("flush temp config");
    file
}

fn config_without_endpoint() -> &'static str {
    indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        bucket = "app-database-backups"
        "#}
}

fn set_var(name: &str, value: &str) {
    // SAFETY: every test here holds ENV_TURN, so these process-global writes do
    // not race another config test, and each test clears what it set.
    unsafe {
        std::env::set_var(name, value);
    }
}

fn clear_var(name: &str) {
    // SAFETY: mirrors `set_var`, run right after the load call under ENV_TURN.
    unsafe {
        std::env::remove_var(name);
    }
}

#[test]
fn a_config_without_an_endpoint_reads_it_from_the_environment() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(config_without_endpoint());
    set_var(ACCESS_KEY_VAR, "test-access-key");
    set_var(SECRET_KEY_VAR, "test-secret-key");
    set_var(ENDPOINT_VAR, ENV_ENDPOINT);

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_var(ACCESS_KEY_VAR);
    clear_var(SECRET_KEY_VAR);
    clear_var(ENDPOINT_VAR);
    assert_eq!(settings.storage.endpoint, ENV_ENDPOINT);
}

#[test]
fn an_endpoint_missing_from_both_the_config_and_the_environment_is_refused() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(config_without_endpoint());
    set_var(ACCESS_KEY_VAR, "test-access-key");
    set_var(SECRET_KEY_VAR, "test-secret-key");
    clear_var(ENDPOINT_VAR);

    let failure = Settings::load(Some(config.path())).expect_err("no endpoint anywhere");

    clear_var(ACCESS_KEY_VAR);
    clear_var(SECRET_KEY_VAR);
    assert!(matches!(failure, ConfigError::MissingEndpoint));
}

#[test]
fn walg_inherits_the_endpoint_resolved_from_the_environment() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        bucket = "app-database-backups"

        [walg]
        s3_prefix = "s3://app-walg/"
        "#});
    set_var(ACCESS_KEY_VAR, "test-access-key");
    set_var(SECRET_KEY_VAR, "test-secret-key");
    set_var(ENDPOINT_VAR, ENV_ENDPOINT);
    set_var(WALG_ACCESS_KEY_VAR, "test-walg-key");
    set_var(WALG_SECRET_KEY_VAR, "test-walg-secret");

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_var(ACCESS_KEY_VAR);
    clear_var(SECRET_KEY_VAR);
    clear_var(ENDPOINT_VAR);
    clear_var(WALG_ACCESS_KEY_VAR);
    clear_var(WALG_SECRET_KEY_VAR);
    let WalgMode::Enabled(walg) = settings.walg else {
        panic!("walg should be enabled");
    };
    assert_eq!(walg.endpoint, ENV_ENDPOINT);
}
