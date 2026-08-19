use super::{StorageCredentials, WalgCredentials};

fn storage() -> StorageCredentials {
    StorageCredentials {
        access_key_id: "AKIAEXAMPLE".to_owned(),
        secret_access_key: "super-secret-value".to_owned(),
    }
}

#[test]
fn a_storage_credential_never_prints_its_secret() {
    // `Settings` derives Debug and holds this, so one `{:?}` or one tracing
    // field added later would otherwise ship the token to wherever logs go.
    let shown = format!("{:?}", storage());

    assert!(!shown.contains("super-secret-value"));
    assert!(shown.contains("<redacted>"));
}

#[test]
fn a_storage_credential_still_shows_the_key_id() {
    // Which credential is loaded is the thing a debug line is read for, and the
    // key id is what tells the right one from the wrong one.
    assert!(format!("{:?}", storage()).contains("AKIAEXAMPLE"));
}

#[test]
fn a_walg_credential_is_redacted_the_same_way() {
    let credentials = WalgCredentials {
        access_key_id: "AKIAWALG".to_owned(),
        secret_access_key: "another-secret".to_owned(),
    };

    let shown = format!("{credentials:?}");

    assert!(!shown.contains("another-secret"));
    assert!(shown.contains("AKIAWALG"));
}

#[test]
fn settings_carrying_a_credential_do_not_leak_it_either() {
    // The realistic leak is not printing the credential directly; it is printing
    // the whole settings struct while diagnosing something else.
    let nested = format!("{:?}", Some(storage()));

    assert!(!nested.contains("super-secret-value"));
}
