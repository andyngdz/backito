//! Moving bytes in and out of the bucket.
//!
//! Uploads go through `put_object_stream`, which chunks into a multipart
//! upload, so peak memory is one chunk regardless of archive size.

use std::path::Path;
use tokio::io::{AsyncRead, AsyncWrite};

use super::operation::ensure_ok;
use super::{ObjectStore, ObjectStoreError, StoreOperation};

impl ObjectStore {
    /// Uploads `reader` to `key` as a multipart stream.
    pub async fn upload_stream<R: AsyncRead + Unpin + ?Sized>(
        &self,
        key: &str,
        reader: &mut R,
    ) -> Result<(), ObjectStoreError> {
        let response = self
            .bucket
            .put_object_stream(reader, key)
            .await
            .map_err(|source| request_failure(StoreOperation::Upload, key, source))?;

        ensure_ok(StoreOperation::Upload, key, response.status_code())
    }

    /// Uploads a small in-memory body, used for the checksum sidecar.
    pub async fn upload_bytes(&self, key: &str, body: &[u8]) -> Result<(), ObjectStoreError> {
        let response = self
            .bucket
            .put_object(key, body)
            .await
            .map_err(|source| request_failure(StoreOperation::Upload, key, source))?;

        ensure_ok(StoreOperation::Upload, key, response.status_code())
    }

    /// Opens `path` and uploads it to `key`.
    ///
    /// `wrap_reader` lets the caller interpose a progress reporter without this
    /// layer knowing anything about how progress is displayed.
    pub async fn upload_file<F, R>(
        &self,
        key: &str,
        path: &Path,
        wrap_reader: F,
    ) -> Result<(), ObjectStoreError>
    where
        F: FnOnce(tokio::fs::File) -> R,
        R: AsyncRead + Unpin,
    {
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|source| local_failure(StoreOperation::Upload, path, source))?;
        let mut reader = wrap_reader(file);
        self.upload_stream(key, &mut reader).await
    }

    /// Streams `key` into `writer`.
    pub async fn download_to<W: AsyncWrite + Send + Unpin + ?Sized>(
        &self,
        key: &str,
        writer: &mut W,
    ) -> Result<(), ObjectStoreError> {
        let status = self
            .bucket
            .get_object_to_writer(key, writer)
            .await
            .map_err(|source| request_failure(StoreOperation::Download, key, source))?;

        ensure_ok(StoreOperation::Download, key, status)
    }

    /// Downloads `key` into a new file at `path`.
    pub async fn download_file(&self, key: &str, path: &Path) -> Result<(), ObjectStoreError> {
        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|source| local_failure(StoreOperation::Download, path, source))?;
        self.download_to(key, &mut file).await
    }

    /// Reads a small object into a string, used for the checksum sidecar.
    pub async fn download_text(&self, key: &str) -> Result<String, ObjectStoreError> {
        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(|source| request_failure(StoreOperation::Download, key, source))?;

        ensure_ok(StoreOperation::Download, key, response.status_code())?;
        Ok(String::from_utf8_lossy(response.as_slice()).into_owned())
    }

    /// Byte size of `key` as the store reports it.
    pub async fn object_size(&self, key: &str) -> Result<u64, ObjectStoreError> {
        let (head, status) = self
            .bucket
            .head_object(key)
            .await
            .map_err(|source| request_failure(StoreOperation::Head, key, source))?;

        ensure_ok(StoreOperation::Head, key, status)?;
        Ok(head.content_length.unwrap_or_default().max(0) as u64)
    }
}

/// Builds the failure for a request that never got a usable answer.
fn request_failure(
    operation: StoreOperation,
    key: &str,
    source: s3::error::S3Error,
) -> ObjectStoreError {
    ObjectStoreError::Request {
        operation: operation.as_str().to_owned(),
        key: key.to_owned(),
        source,
    }
}

/// Builds the failure for a local file backing a transfer.
fn local_failure(
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
