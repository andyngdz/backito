use super::{StoreOperation, ensure_ok};
use crate::infra::object_store::ObjectStoreError;

#[test]
fn every_operation_has_a_name() {
    assert_eq!(StoreOperation::List.as_str(), "list");
    assert_eq!(StoreOperation::Upload.as_str(), "upload");
    assert_eq!(StoreOperation::Download.as_str(), "download");
    assert_eq!(StoreOperation::Head.as_str(), "head");
}

#[test]
fn http_200_passes() {
    assert!(ensure_ok(StoreOperation::Upload, "some.dump", 200).is_ok());
}

#[test]
fn a_403_names_the_operation_and_key() {
    let failure = ensure_ok(StoreOperation::Upload, "some.dump", 403).expect_err("403 must fail");

    assert!(matches!(
        failure,
        ObjectStoreError::Status { status: 403, ref key, .. } if key == "some.dump"
    ));
    assert!(failure.to_string().contains("upload"));
}

#[test]
fn a_206_partial_response_is_not_accepted_as_success() {
    // A ranged or truncated response would silently produce a short archive.
    assert!(ensure_ok(StoreOperation::Download, "some.dump", 206).is_err());
}
