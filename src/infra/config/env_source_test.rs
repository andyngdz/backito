use super::EnvSource;
use crate::infra::config::{
    ConfigCore, ConfigError, ConfigSource, ContainerSource, ENV_TURN, WalgMode,
};

/// Every variable this source reads, so a test starts from a known-empty
/// environment rather than from whatever the previous one left behind.
const READ_VARS: [&str; 17] = [
    "BACKITO_DB_LABEL",
    "BACKITO_DB_CONTAINER",
    "BACKITO_DB_SERVICE",
    "BACKITO_DB_CONTAINER_LABEL",
    "BACKITO_DB_NAME",
    "BACKITO_DB_USER",
    "BACKITO_DB_IMAGE",
    "BACKITO_DB_RESTORE_JOBS",
    "BACKITO_ENDPOINT",
    "BACKITO_BUCKET",
    "BACKITO_REGION",
    "BACKITO_BACKUP_INTERVAL",
    "BACKITO_VERIFY_INTERVAL",
    "BACKITO_RETAIN",
    "BACKITO_WALG_S3_PREFIX",
    "BACKITO_WALG_BASE_INTERVAL",
    "BACKITO_WALG_RETAIN_FULL",
];

fn put(name: &str, value: &str) {
    // SAFETY: guarded by ENV_TURN, so no other test reads these concurrently.
    unsafe { std::env::set_var(name, value) }
}

fn clear() {
    for name in READ_VARS {
        // SAFETY: mirrors `put`, inside the same guarded section.
        unsafe { std::env::remove_var(name) }
    }
}

/// The smallest environment that loads: everything else has a default.
fn set_required() {
    put("BACKITO_DB_LABEL", "app");
    put("BACKITO_DB_CONTAINER", "app-db");
    put("BACKITO_DB_NAME", "postgres");
    put("BACKITO_DB_IMAGE", "postgres:17");
    put(
        "BACKITO_ENDPOINT",
        "https://account.r2.cloudflarestorage.com",
    );
    put("BACKITO_BUCKET", "app-database-backups");
}

fn load() -> Result<ConfigCore, ConfigError> {
    EnvSource.load()
}

#[test]
fn the_required_variables_are_enough_and_the_rest_default() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_required();

    let core = load().expect("load");

    clear();
    assert_eq!(
        core.database.container,
        ContainerSource::Named("app-db".to_owned())
    );
    assert_eq!(core.database.user, "postgres");
    assert_eq!(core.database.restore_jobs, 4);
    assert_eq!(core.storage.region, "auto");
    assert_eq!(core.schedule.backup_interval.as_secs(), 24 * 60 * 60);
    assert_eq!(core.walg, WalgMode::Disabled);
}

#[test]
fn a_missing_required_variable_names_itself() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_required();
    // SAFETY: guarded by ENV_TURN, cleared straight after the load.
    unsafe { std::env::remove_var("BACKITO_BUCKET") }

    let failure = load().expect_err("the bucket is required");

    clear();
    assert!(matches!(
        failure,
        ConfigError::MissingEnvVar { ref variable } if variable == "BACKITO_BUCKET"
    ));
}

#[test]
fn a_service_resolves_against_the_label_the_orchestrator_uses() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_required();
    // SAFETY: guarded by ENV_TURN, cleared straight after the load.
    unsafe { std::env::remove_var("BACKITO_DB_CONTAINER") }
    put("BACKITO_DB_SERVICE", "db");
    put("BACKITO_DB_CONTAINER_LABEL", "uncloud.service.name");

    let core = load().expect("load");

    clear();
    assert_eq!(
        core.database.container,
        ContainerSource::Service {
            label: "uncloud.service.name".to_owned(),
            service: "db".to_owned(),
        }
    );
}

#[test]
fn the_wal_prefix_is_what_turns_archiving_on() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_required();
    put("BACKITO_WALG_S3_PREFIX", "s3://app-walg/");
    put("BACKITO_WALG_BASE_INTERVAL", "12h");
    put("BACKITO_WALG_RETAIN_FULL", "2");

    let core = load().expect("load");

    clear();
    let WalgMode::Enabled(walg) = core.walg else {
        panic!("expected wal-g to be configured");
    };
    assert_eq!(walg.s3_prefix, "s3://app-walg/");
    // The endpoint is account-level, so it falls back within this same source.
    assert_eq!(walg.endpoint, "https://account.r2.cloudflarestorage.com");
    assert_eq!(walg.base_interval.as_secs(), 12 * 60 * 60);
    assert_eq!(walg.retain_full, 2);
}

#[test]
fn an_unreadable_interval_names_the_variable_it_came_from() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_required();
    put("BACKITO_BACKUP_INTERVAL", "every day");

    let failure = load().expect_err("not an interval");

    clear();
    assert!(
        failure.to_string().contains("BACKITO_BACKUP_INTERVAL"),
        "the message should name the variable, got: {failure}"
    );
}
