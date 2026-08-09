use super::{WalgFile, WalgMode};
use crate::infra::config::{ConfigCore, ConfigError, ConfigSource, TomlSource};
use indoc::indoc;
use std::io::Write;
use tempfile::NamedTempFile;

fn load(body: &str) -> Result<ConfigCore, ConfigError> {
    let mut file = NamedTempFile::new().expect("create temp config");
    file.write_all(body.as_bytes()).expect("write temp config");
    file.flush().expect("flush temp config");
    TomlSource::new(Some(file.path())).load()
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
    let core = load(without_walg()).expect("load");

    assert_eq!(core.walg, WalgMode::Disabled);
}

#[test]
fn a_walg_table_is_read_and_defaults_the_endpoint_to_the_archive_store() {
    let core = load(with_walg()).expect("load");

    let WalgMode::Enabled(walg) = core.walg else {
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
fn an_explicit_walg_endpoint_wins_over_the_archive_stores() {
    let core = load(indoc! {r#"
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
        endpoint = "https://other.example.com"
        "#})
    .expect("load");

    let WalgMode::Enabled(walg) = core.walg else {
        panic!("expected wal-g to be configured");
    };
    assert_eq!(walg.endpoint, "https://other.example.com");
}

#[test]
fn an_unreadable_base_interval_names_its_field() {
    let failure = load(indoc! {r#"
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
        "#})
    .expect_err("not an interval");

    assert!(failure.to_string().contains("base_interval"));
}

#[test]
fn the_file_type_is_reachable_for_callers_that_build_one_directly() {
    // Guards the export: WalgFile is the parsed shape, and dropping it from the
    // module would be a silent API change rather than a compile error.
    fn assert_deserialize<T: serde::de::DeserializeOwned>() {}
    assert_deserialize::<WalgFile>();
}
