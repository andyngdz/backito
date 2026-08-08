use super::ContainerSource;
use super::{ConfigError, Settings};
use indoc::indoc;
use std::io::Write;
use tempfile::NamedTempFile;

/// The credential variables are process-global, so these tests take turns.
/// Without this they race: one clears what another just set.
static ENV_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());

const ACCESS_KEY_VAR: &str = "BACKITO_ACCESS_KEY_ID";
const SECRET_KEY_VAR: &str = "BACKITO_SECRET_ACCESS_KEY";

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

fn set_credentials(access_key: &str) {
    // SAFETY: the config tests run in one process and set both variables around
    // a single load call.
    unsafe {
        std::env::set_var(ACCESS_KEY_VAR, access_key);
        std::env::set_var(SECRET_KEY_VAR, "test-secret-key");
    }
}

fn clear_credentials() {
    // SAFETY: mirrors `set_credentials`, immediately after the load call.
    unsafe {
        std::env::remove_var(ACCESS_KEY_VAR);
        std::env::remove_var(SECRET_KEY_VAR);
    }
}

#[test]
fn full_config_loads_every_field() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(full_config());
    set_credentials("test-access-key");

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    assert_eq!(settings.database.label, "app");
    assert_eq!(
        settings.database.container,
        ContainerSource::Named("app-db".to_owned())
    );
    assert_eq!(settings.database.restore_jobs, 1);
    assert_eq!(settings.storage.bucket, "app-database-backups");
    assert_eq!(settings.credentials.access_key_id, "test-access-key");
}

#[test]
fn user_and_region_fall_back_to_postgres_and_auto() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(minimal_config());
    set_credentials("test-access-key");

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    assert_eq!(settings.database.user, "postgres");
    assert_eq!(settings.database.restore_jobs, 4);
    assert_eq!(settings.storage.region, "auto");
}

#[test]
fn a_blank_credential_is_treated_as_missing() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(full_config());
    set_credentials("   ");

    let failure = Settings::load(Some(config.path())).expect_err("blank must fail");

    clear_credentials();
    assert!(matches!(
        failure,
        ConfigError::MissingCredential { ref variable } if variable == ACCESS_KEY_VAR
    ));
}

#[test]
fn a_missing_file_names_the_path_it_tried() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    set_credentials("test-access-key");

    let failure = Settings::load(Some(std::path::Path::new("/nonexistent/backito.toml")))
        .expect_err("missing file must fail");

    clear_credentials();
    assert!(matches!(failure, ConfigError::ReadFile { .. }));
}

#[test]
fn a_service_resolves_against_the_compose_label_by_default() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(indoc! {r#"
        [database]
        label = "app"
        service = "db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#});
    set_credentials("test-access-key");

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    assert_eq!(
        settings.database.container,
        ContainerSource::Service {
            label: "com.docker.compose.service".to_owned(),
            service: "db".to_owned(),
        }
    );
}

#[test]
fn container_label_overrides_the_default_for_other_orchestrators() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(indoc! {r#"
        [database]
        label = "app"
        service = "db"
        container_label = "uncloud.service.name"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#});
    set_credentials("test-access-key");

    let settings = Settings::load(Some(config.path())).expect("load");

    clear_credentials();
    assert_eq!(
        settings.database.container,
        ContainerSource::Service {
            label: "uncloud.service.name".to_owned(),
            service: "db".to_owned(),
        }
    );
}

#[test]
fn naming_both_a_container_and_a_service_is_refused() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(indoc! {r#"
        [database]
        label = "app"
        container = "app-db"
        service = "db"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#});
    set_credentials("test-access-key");

    let failure = Settings::load(Some(config.path())).expect_err("both cannot hold");

    clear_credentials();
    assert!(matches!(failure, ConfigError::ContainerOverSpecified));
}

#[test]
fn naming_neither_a_container_nor_a_service_is_refused() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = write_config(indoc! {r#"
        [database]
        label = "app"
        name = "postgres"
        image = "postgres:17"

        [storage]
        endpoint = "https://account.r2.cloudflarestorage.com"
        bucket = "app-database-backups"
        "#});
    set_credentials("test-access-key");

    let failure = Settings::load(Some(config.path())).expect_err("one of the two is required");

    clear_credentials();
    assert!(matches!(failure, ConfigError::ContainerUnspecified));
}
