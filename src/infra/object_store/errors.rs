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
        /// Underlying client failure. Boxed because this error is large
        /// enough to bloat every Result that carries it up the call stack.
        source: Box<object_store::Error>,
    },

    /// A request failed at the transport or protocol level.
    ///
    /// This carries the store's own error rather than an HTTP status, because
    /// the client already separates not-found, denied, and transport failures;
    /// rebuilding that from a status code would lose detail.
    #[error("{operation} {key}: {source}")]
    Request {
        /// What was being attempted, e.g. `upload`.
        operation: String,
        /// Object key involved.
        key: String,
        /// Underlying client failure. Boxed because this error is large
        /// enough to bloat every Result that carries it up the call stack.
        source: Box<object_store::Error>,
    },

    /// The local end of a transfer failed while bytes were moving.
    #[error("{operation} {key}: {source}")]
    LocalStream {
        /// What was being attempted.
        operation: String,
        /// Object key involved.
        key: String,
        /// Underlying io failure.
        source: std::io::Error,
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

impl ObjectStoreError {
    /// True when the store answered and said the key is not there.
    ///
    /// Callers need this separated from every other request failure: a missing
    /// object is an answer about the bucket's contents, while a refused or
    /// unreachable request says nothing about them. This lives here so the rest
    /// of the tool never has to match on the storage client's own error type.
    pub fn is_missing_object(&self) -> bool {
        matches!(
            self,
            Self::Request { source, .. } if matches!(**source, object_store::Error::NotFound { .. })
        )
    }
}

#[cfg(test)]
#[path = "errors_test.rs"]
mod errors_test;
