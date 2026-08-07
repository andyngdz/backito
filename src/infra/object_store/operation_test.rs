use super::StoreOperation;

#[test]
fn every_operation_has_a_name() {
    assert_eq!(StoreOperation::List.as_str(), "list");
    assert_eq!(StoreOperation::Upload.as_str(), "upload");
    assert_eq!(StoreOperation::Download.as_str(), "download");
    assert_eq!(StoreOperation::Head.as_str(), "head");
}
