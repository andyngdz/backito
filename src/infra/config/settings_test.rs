use super::{Settings, check_endpoint};
use crate::domain::Interval;
use crate::infra::config::{
    ConfigCore, ConfigError, ConfigSource, ContainerSource, DatabaseSettings, ScheduleSettings,
    SecretSource, Secrets, StorageCredentials, StorageSettings, WalgCredentials, WalgMode,
    WalgSettings,
};

/// A config source that hands back what it was built with, so the assembly can
/// be tested without a file on disk or a variable in the environment.
struct FixedConfig(ConfigCore);

impl ConfigSource for FixedConfig {
    fn load(&self) -> Result<ConfigCore, ConfigError> {
        Ok(self.0.clone())
    }
}

/// The secret half of the same fixture.
struct FixedSecrets(Secrets);

impl SecretSource for FixedSecrets {
    fn load(&self) -> Result<Secrets, ConfigError> {
        Ok(self.0.clone())
    }
}

fn core(walg: WalgMode) -> ConfigCore {
    ConfigCore {
        database: DatabaseSettings {
            label: "app".to_owned(),
            container: ContainerSource::Named("app-db".to_owned()),
            name: "postgres".to_owned(),
            user: "postgres".to_owned(),
            image: "postgres:17".to_owned(),
            restore_jobs: 4,
        },
        storage: StorageSettings {
            endpoint: "https://account.r2.cloudflarestorage.com".to_owned(),
            bucket: "app-database-backups".to_owned(),
            region: "auto".to_owned(),
        },
        schedule: ScheduleSettings::default(),
        walg,
    }
}

fn walg_enabled() -> WalgMode {
    WalgMode::Enabled(Box::new(WalgSettings {
        s3_prefix: "s3://app-walg/".to_owned(),
        endpoint: "https://account.r2.cloudflarestorage.com".to_owned(),
        region: "auto".to_owned(),
        data_dir: "/var/lib/postgresql/data".to_owned(),
        base_interval: Interval::from_secs(24 * 60 * 60),
        retain_full: 3,
        binary: "wal-g".to_owned(),
    }))
}

fn secrets(walg: Option<WalgCredentials>) -> Secrets {
    Secrets {
        storage: StorageCredentials {
            access_key_id: "test-access-key".to_owned(),
            secret_access_key: "test-secret-key".to_owned(),
        },
        walg,
    }
}

fn walg_credentials() -> WalgCredentials {
    WalgCredentials {
        access_key_id: "walg-access-key".to_owned(),
        secret_access_key: "walg-secret-key".to_owned(),
    }
}

#[test]
fn load_takes_one_source_of_each_kind_and_neither_fills_the_other() {
    let settings = Settings::load(
        &FixedConfig(core(WalgMode::Disabled)),
        &FixedSecrets(secrets(None)),
    )
    .expect("load");

    assert_eq!(settings.storage.bucket, "app-database-backups");
    assert_eq!(settings.credentials.secret_access_key, "test-secret-key");
}

#[test]
fn the_two_halves_join_into_one_configuration() {
    let settings = Settings::from_parts(core(WalgMode::Disabled), secrets(None)).expect("assemble");

    assert_eq!(settings.database.label, "app");
    assert_eq!(settings.storage.bucket, "app-database-backups");
    assert_eq!(settings.credentials.access_key_id, "test-access-key");
    assert_eq!(settings.walg, WalgMode::Disabled);
}

#[test]
fn wal_archiving_hands_back_its_settings_and_token_together() {
    let settings = Settings::from_parts(core(walg_enabled()), secrets(Some(walg_credentials())))
        .expect("assemble");

    let (walg, credentials) = settings.walg_runtime().expect("wal archiving is on");
    assert_eq!(walg.s3_prefix, "s3://app-walg/");
    assert_eq!(credentials.access_key_id, "walg-access-key");
}

#[test]
fn wal_archiving_without_its_token_is_refused() {
    let failure =
        Settings::from_parts(core(walg_enabled()), secrets(None)).expect_err("token required");

    assert!(matches!(failure, ConfigError::MissingWalgCredentials));
}

#[test]
fn a_wal_token_without_wal_archiving_is_dropped() {
    let settings =
        Settings::from_parts(core(WalgMode::Disabled), secrets(Some(walg_credentials())))
            .expect("assemble");

    assert!(settings.walg_runtime().is_none());
}

#[test]
fn the_placeholder_endpoint_init_writes_is_refused_with_a_sentence() {
    // `object_store` panics on an unparsable URI rather than returning, so
    // without this the first command after `init` ends in a library stack trace.
    let failure = check_endpoint("https://<account-id>.r2.cloudflarestorage.com")
        .expect_err("the placeholder must not load");

    assert!(matches!(failure, ConfigError::UnusableEndpoint { .. }));
}

#[test]
fn a_real_endpoint_loads() {
    for usable in [
        "https://abc123.r2.cloudflarestorage.com",
        "https://s3.eu-central-1.amazonaws.com",
        "http://127.0.0.1:9000",
        "http://minio:9000",
    ] {
        assert!(check_endpoint(usable).is_ok(), "{usable} should load");
    }
}

#[test]
fn an_endpoint_with_no_scheme_or_no_host_is_refused() {
    for unusable in [
        "r2.cloudflarestorage.com",
        "s3://bucket",
        "https://",
        "",
        "https://my endpoint.com",
    ] {
        assert!(
            check_endpoint(unusable).is_err(),
            "{unusable} should be refused"
        );
    }
}
