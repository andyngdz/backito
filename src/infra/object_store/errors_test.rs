use super::ObjectStoreError;

/// Builds a request failure carrying `source`.
fn request_failure(source: object_store::Error) -> ObjectStoreError {
    ObjectStoreError::Request {
        operation: "download".to_owned(),
        key: "app-backup-20260803-0942.dump".to_owned(),
        source: Box::new(source),
    }
}

#[test]
fn a_missing_key_is_reported_as_missing() {
    let failure = request_failure(object_store::Error::NotFound {
        path: "app-backup-20260803-0942.dump".to_owned(),
        source: "no such key".into(),
    });

    assert!(failure.is_missing_object());
}

#[test]
fn a_refused_request_is_not_a_missing_key() {
    // Treating this as missing would report "no checksum stored" for an archive
    // whose sidecar is present and simply out of reach.
    let failure = request_failure(object_store::Error::PermissionDenied {
        path: "app-backup-20260803-0942.dump".to_owned(),
        source: "denied".into(),
    });

    assert!(!failure.is_missing_object());
}

#[test]
fn an_empty_bucket_is_not_a_missing_key() {
    let failure = ObjectStoreError::NoArchives {
        bucket: "app-database-backups".to_owned(),
    };

    assert!(!failure.is_missing_object());
}
