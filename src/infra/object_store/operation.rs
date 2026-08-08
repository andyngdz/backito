//! Names the object-store operations, so one failed request says which.

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
    /// Remove an object.
    Delete,
}

impl StoreOperation {
    /// The name used in error messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Upload => "upload",
            Self::Download => "download",
            Self::Head => "head",
            Self::Delete => "delete",
        }
    }
}

#[cfg(test)]
#[path = "operation_test.rs"]
mod operation_test;
