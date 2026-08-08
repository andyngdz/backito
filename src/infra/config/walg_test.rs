use super::{WalgFile, WalgMode};
use crate::infra::config::Settings;
use indoc::indoc;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::infra::config::ENV_TURN;

fn write_config(body: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp config");
    file.write_all(body.as_bytes()).expect("write temp config");
    file.flush().expect("flush temp config");
    file
}

fn set_credentials() {
    // SAFETY: the config tests run in one process and set every variable around
    // a single load call.
    unsafe {
        std::env::set_var("BACKITO_ACCESS_KEY_ID", "archive-key");
        std::env::set_var("BACKITO_SECRET_ACCESS_KEY", "archive-secret");
        std::env::set_var("BACKITO_WALG_ACCESS_KEY_ID", "walg-key");
        std::env::set_var("BACKITO_WALG_SECRET_ACCESS_KEY", "walg-secret");
    }
}

fn clear_credentials() {
    // SAFETY: mirrors `set_credentials`, immediately after the load call.
    unsafe {
        std::env::remove_var("BACKITO_ACCESS_KEY_ID");
        std::env::remove_var("BACKITO_SECRET_ACCESS_KEY");
        std::env::remove_var("BACKITO_WALG_ACCESS_KEY_ID");
        std::env::remove_var("BACKITO_WALG_SECRET_ACCESS_KEY");
    }
}

fn without_walg() -> &'static str {
    indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#}
}

fn with_walg() -> &'static str {
    indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"

        [walg]
        s3_prefix = "s3://app-walg/"
        base_interval = "12h"
        retain_full = 2
        "#}
}

#[test]
fn a_config_without_a_walg_table_takes_no_physical_backups() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(without_walg());
    set_credentials();

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    assert_eq!(settings.walg, WalgMode::Disabled);
}

#[test]
fn a_walg_table_is_read_and_defaults_the_endpoint_to_the_archive_store() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(with_walg());
    set_credentials();

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    let WalgMode::Enabled(walg) = settings.walg else {
        panic!("expected wal-g to be configured");
    };
    assert_eq!(walg.s3_prefix, "s3://app-walg/");
    // The endpoint is account-level, so only the prefix and the token differ.
    assert_eq!(walg.endpoint, "https://account.r2.cloudflarestorage.com");
    assert_eq!(walg.base_interval.as_secs(), 12 * 60 * 60);
    assert_eq!(walg.retain_full, 2);
    assert_eq!(walg.data_dir, "/var/lib/postgresql/data");
    assert_eq!(walg.binary, "wal-g");
}

#[test]
fn wal_storage_uses_its_own_credentials_not_the_archive_stores() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(with_walg());
    set_credentials();

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    let WalgMode::Enabled(walg) = settings.walg else {
        panic!("expected wal-g to be configured");
    };
    // A bucket-scoped token is only a boundary while the two stay apart.
    assert_eq!(walg.credentials.access_key_id, "walg-key");
    assert_eq!(settings.credentials.access_key_id, "archive-key");
}

#[test]
fn a_missing_wal_credential_names_the_variable_to_export() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(with_walg());
    set_credentials();
    // SAFETY: removed and restored inside the same guarded section.
    unsafe {
        std::env::remove_var("BACKITO_WALG_ACCESS_KEY_ID");
    }

    let failure = Settings::load(Some(config.path())).expect_err("no wal-g key");

    clear_credentials();
    assert!(
        failure.to_string().contains("BACKITO_WALG_ACCESS_KEY_ID"),
        "the message should name the variable, got: {failure}"
    );
}

#[test]
fn an_unreadable_base_interval_names_its_field() {
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
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"

        [walg]
        s3_prefix = "s3://app-walg/"
        base_interval = "nightly"
        "#});
    set_credentials();

    let failure = Settings::load(Some(config.path())).expect_err("not an interval");

    clear_credentials();
    assert!(failure.to_string().contains("base_interval"));
}

#[test]
fn the_file_type_is_reachable_for_callers_that_build_one_directly() {
    // Guards the export: WalgFile is the parsed shape, and dropping it from the
    // module would be a silent API change rather than a compile error.
    fn assert_deserialize<T: serde::de::DeserializeOwned>() {}
    assert_deserialize::<WalgFile>();
}
