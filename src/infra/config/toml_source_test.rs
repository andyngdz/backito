use super::TomlSource;
use crate::infra::config::{
    ConfigCore, ConfigError, ConfigSource, ContainerSource, ScheduleSettings,
};
use indoc::indoc;
use std::io::Write;
use tempfile::NamedTempFile;

fn full_config() -> &'static str {
    indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        user = "postgres"
        image = "postgres:17"
        restore_jobs = 1

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        region = "auto"
        "#}
}

fn minimal_config() -> &'static str {
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

fn write_config(body: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp config");
    file.write_all(body.as_bytes()).expect("write temp config");
    file.flush().expect("flush temp config");
    file
}

fn load(body: &str) -> Result<ConfigCore, ConfigError> {
    let file = write_config(body);
    TomlSource::new(Some(file.path())).load()
}

#[test]
fn full_config_loads_every_field() {
    let core = load(full_config()).expect("load");

    assert_eq!(core.database.label, "app");
    assert_eq!(
        core.database.container,
        ContainerSource::Named("app-db".to_owned())
    );
    assert_eq!(core.database.restore_jobs, 1);
    assert_eq!(core.storage.bucket, "app-database-backups");
    assert_eq!(
        core.storage.endpoint,
        "https://account.r2.cloudflarestorage.com"
    );
}

#[test]
fn user_and_region_fall_back_to_postgres_and_auto() {
    let core = load(minimal_config()).expect("load");

    assert_eq!(core.database.user, "postgres");
    assert_eq!(core.database.restore_jobs, 4);
    assert_eq!(core.storage.region, "auto");
}

#[test]
fn a_missing_file_names_the_path_it_tried() {
    let failure = TomlSource::new(Some(std::path::Path::new("/nonexistent/backito.toml")))
        .load()
        .expect_err("missing file must fail");

    assert!(matches!(failure, ConfigError::ReadFile { .. }));
}

#[test]
fn a_file_without_an_endpoint_is_incomplete() {
    let failure = load(indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        bucket = "app-database-backups"
        "#})
    .expect_err("the endpoint is required in a file");

    assert!(matches!(failure, ConfigError::ParseFile { .. }));
}

#[test]
fn a_service_resolves_against_the_compose_label_by_default() {
    let core = load(indoc! {r#"
        [database]
        label = "app"
        service = "db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#})
    .expect("load");

    assert_eq!(
        core.database.container,
        ContainerSource::Service {
            label: "com.docker.compose.service".to_owned(),
            service: "db".to_owned(),
        }
    );
}

#[test]
fn container_label_overrides_the_default_for_other_orchestrators() {
    let core = load(indoc! {r#"
        [database]
        label = "app"
        service = "db"
        container_label = "uncloud.service.name"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#})
    .expect("load");

    assert_eq!(
        core.database.container,
        ContainerSource::Service {
            label: "uncloud.service.name".to_owned(),
            service: "db".to_owned(),
        }
    );
}

#[test]
fn naming_both_a_container_and_a_service_is_refused() {
    let failure = load(indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        service = "db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#})
    .expect_err("both cannot hold");

    assert!(matches!(failure, ConfigError::ContainerOverSpecified));
}

#[test]
fn naming_neither_a_container_nor_a_service_is_refused() {
    let failure = load(indoc! {r#"
        [database]
        label = "app"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#})
    .expect_err("one of the two is required");

    assert!(matches!(failure, ConfigError::ContainerUnspecified));
}

#[test]
fn a_config_without_a_schedule_table_gets_the_defaults() {
    let core = load(minimal_config()).expect("load");

    assert_eq!(core.schedule, ScheduleSettings::default());
    assert_eq!(core.schedule.backup_interval.as_secs(), 24 * 60 * 60);
    assert_eq!(core.schedule.verify_interval.as_secs(), 7 * 24 * 60 * 60);
    assert_eq!(core.schedule.retain, 7);
}

#[test]
fn a_schedule_table_is_read_in_human_units() {
    let core = load(indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"

        [schedule]
        backup_interval = "6h"
        verify_interval = "0s"
        retain = 3
        "#})
    .expect("load");

    assert_eq!(core.schedule.backup_interval.as_secs(), 6 * 60 * 60);
    assert!(core.schedule.verify_interval.is_disabled());
    assert_eq!(core.schedule.retain, 3);
}

#[test]
fn an_unreadable_interval_names_the_field_it_came_from() {
    let failure = load(indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"

        [schedule]
        backup_interval = "every day"
        "#})
    .expect_err("not an interval");

    assert!(
        failure.to_string().contains("backup_interval"),
        "the message should name the field, got: {failure}"
    );
}
