use super::{
    ACCESS_KEY_VAR, EnvSecretSource, SECRET_KEY_VAR, WALG_ACCESS_KEY_VAR, WALG_SECRET_KEY_VAR,
};
use crate::infra::config::{ConfigError, ENV_TURN, SecretSource};

/// Sets `name`, or removes it when the value is `None`.
///
/// The credential variables are process-global, so every test here takes
/// `ENV_TURN` first and clears what it set before releasing it.
fn put(name: &str, value: Option<&str>) {
    // SAFETY: guarded by ENV_TURN, so no other test reads these concurrently.
    unsafe {
        match value {
            Some(text) => std::env::set_var(name, text),
            None => std::env::remove_var(name),
        }
    }
}

fn clear() {
    for name in [
        ACCESS_KEY_VAR,
        SECRET_KEY_VAR,
        WALG_ACCESS_KEY_VAR,
        WALG_SECRET_KEY_VAR,
    ] {
        put(name, None);
    }
}

fn set_archive() {
    put(ACCESS_KEY_VAR, Some("archive-key"));
    put(SECRET_KEY_VAR, Some("archive-secret"));
}

#[test]
fn the_archive_token_is_read_from_the_environment() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_archive();

    let secrets = EnvSecretSource.load().expect("load");

    clear();
    assert_eq!(secrets.storage.access_key_id, "archive-key");
    assert_eq!(secrets.storage.secret_access_key, "archive-secret");
    assert!(secrets.walg.is_none());
}

#[test]
fn a_blank_archive_token_is_treated_as_missing() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    put(ACCESS_KEY_VAR, Some("   "));
    put(SECRET_KEY_VAR, Some("archive-secret"));

    let failure = EnvSecretSource.load().expect_err("blank must fail");

    clear();
    assert!(matches!(
        failure,
        ConfigError::MissingEnvVar { ref variable } if variable == ACCESS_KEY_VAR
    ));
}

#[test]
fn the_wal_token_is_read_when_both_names_are_set() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_archive();
    put(WALG_ACCESS_KEY_VAR, Some("walg-key"));
    put(WALG_SECRET_KEY_VAR, Some("walg-secret"));

    let secrets = EnvSecretSource.load().expect("load");

    clear();
    let walg = secrets.walg.expect("wal token");
    // A bucket-scoped token is only a boundary while the two stay apart.
    assert_eq!(walg.access_key_id, "walg-key");
    assert_eq!(secrets.storage.access_key_id, "archive-key");
}

#[test]
fn half_a_wal_token_names_the_variable_still_missing() {
    let _turn = ENV_TURN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear();
    set_archive();
    put(WALG_ACCESS_KEY_VAR, Some("walg-key"));

    let failure = EnvSecretSource.load().expect_err("half a token is a typo");

    clear();
    assert!(
        failure.to_string().contains(WALG_SECRET_KEY_VAR),
        "the message should name the variable, got: {failure}"
    );
}
