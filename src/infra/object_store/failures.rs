//! Building the failures a transfer can raise.
//!
//! Shared by the upload and download halves so one operation is never described
//! two ways in a message an operator has to act on.

use std::path::Path;

use super::{ObjectStoreError, StoreOperation};

/// Builds the failure for a request that never got a usable answer.
pub fn request_failure(
    operation: StoreOperation,
    key: &str,
    source: object_store::Error,
) -> ObjectStoreError {
    ObjectStoreError::Request {
        operation: operation.as_str().to_owned(),
        key: key.to_owned(),
        source: Box::new(source),
    }
}

/// Builds the failure for reading the local side of an upload.
pub fn read_failure(key: &str, source: std::io::Error) -> ObjectStoreError {
    ObjectStoreError::LocalStream {
        operation: StoreOperation::Upload.as_str().to_owned(),
        key: key.to_owned(),
        source,
    }
}

/// Builds the failure for writing the local side of a download.
pub fn write_failure(key: &str, source: std::io::Error) -> ObjectStoreError {
    ObjectStoreError::LocalStream {
        operation: StoreOperation::Download.as_str().to_owned(),
        key: key.to_owned(),
        source,
    }
}

/// Builds the failure for a local file backing a transfer.
pub fn local_failure(
    operation: StoreOperation,
    path: &Path,
    source: std::io::Error,
) -> ObjectStoreError {
    ObjectStoreError::LocalFile {
        operation: operation.as_str().to_owned(),
        path: path.to_string_lossy().into_owned(),
        source,
    }
}
