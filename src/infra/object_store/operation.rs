//! Names the object-store operations, so one failed request says which.

use super::ObjectStoreError;

/// HTTP status a successful object request returns.
const HTTP_OK: u16 = 200;

/// The object-store operations this tool performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOperation {
    /// Enumerate keys in the bucket.
    List,
    /// Write an object.
    Upload,
    /// Read an object.
    Download,
    /// Read an object's metadata.
    Head,
}

impl StoreOperation {
    /// The name used in error messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Upload => "upload",
            Self::Download => "download",
            Self::Head => "head",
        }
    }
}

/// Turns a non-200 status into a typed failure.
pub fn ensure_ok(
    operation: StoreOperation,
    key: &str,
    status: u16,
) -> Result<(), ObjectStoreError> {
    if status == HTTP_OK {
        return Ok(());
    }
    Err(ObjectStoreError::Status {
        operation: operation.as_str().to_owned(),
        key: key.to_owned(),
        status,
    })
}

#[cfg(test)]
#[path = "operation_test.rs"]
mod operation_test;
