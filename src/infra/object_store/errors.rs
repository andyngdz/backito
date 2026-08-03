//! Failures raised while talking to the object store.

use thiserror::Error;

/// Why an object-store operation failed.
#[derive(Debug, Error)]
pub enum ObjectStoreError {
    /// The bucket handle could not be built from the configured endpoint.
    #[error("configure bucket {bucket}: {source}")]
    Configure {
        /// Bucket that was being addressed.
        bucket: String,
        /// Underlying client failure.
        source: s3::error::S3Error,
    },

    /// A request failed at the transport or protocol level.
    #[error("{operation} {key}: {source}")]
    Request {
        /// What was being attempted, e.g. `upload`.
        operation: String,
        /// Object key involved.
        key: String,
        /// Underlying client failure.
        source: s3::error::S3Error,
    },

    /// The service answered, but with a status this tool cannot accept.
    #[error("{operation} {key} returned HTTP {status}")]
    Status {
        /// What was being attempted.
        operation: String,
        /// Object key involved.
        key: String,
        /// HTTP status returned.
        status: u16,
    },

    /// A local file backing an upload or download could not be used.
    #[error("{operation} local file {path}: {source}")]
    LocalFile {
        /// What was being attempted.
        operation: String,
        /// Path involved.
        path: String,
        /// Underlying io failure.
        source: std::io::Error,
    },

    /// The bucket holds no archive to work from.
    #[error("bucket {bucket} holds no backup archive yet -- run `backito backup` first")]
    NoArchives {
        /// Bucket that was searched.
        bucket: String,
    },
}
