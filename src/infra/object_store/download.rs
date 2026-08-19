//! Fetching bytes back out of the bucket.

use std::path::Path;

use futures::StreamExt;
use object_store::ObjectStoreExt as _;
use object_store::path::Path as ObjectPath;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::failures::{local_failure, request_failure, write_failure};
use super::{ObjectStore, ObjectStoreError, StoreOperation};

impl ObjectStore {
    /// Streams `key` into `writer`.
    pub async fn download_to<W: AsyncWrite + Send + Unpin + ?Sized>(
        &self,
        key: &str,
        writer: &mut W,
    ) -> Result<(), ObjectStoreError> {
        let location = ObjectPath::from(key);
        let response = self
            .bucket
            .get(&location)
            .await
            .map_err(|source| request_failure(StoreOperation::Download, key, source))?;

        let mut body = response.into_stream();
        while let Some(chunk) = body.next().await {
            let chunk =
                chunk.map_err(|source| request_failure(StoreOperation::Download, key, source))?;
            writer
                .write_all(&chunk)
                .await
                .map_err(|source| write_failure(key, source))?;
        }

        writer
            .flush()
            .await
            .map_err(|source| write_failure(key, source))?;

        Ok(())
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
        let location = ObjectPath::from(key);
        let response = self
            .bucket
            .get(&location)
            .await
            .map_err(|source| request_failure(StoreOperation::Download, key, source))?;

        let body = response
            .bytes()
            .await
            .map_err(|source| request_failure(StoreOperation::Download, key, source))?;

        Ok(String::from_utf8_lossy(&body).into_owned())
    }

    /// Byte size of `key` as the store reports it.
    pub async fn object_size(&self, key: &str) -> Result<u64, ObjectStoreError> {
        let location = ObjectPath::from(key);
        let meta = self
            .bucket
            .head(&location)
            .await
            .map_err(|source| request_failure(StoreOperation::Head, key, source))?;

        Ok(meta.size)
    }

    /// Removes `key`.
    ///
    /// Retention lives here rather than in a shell wrapper around rclone,
    /// because the credential, the endpoint and the bucket are already settled
    /// on this side and duplicating them is how the two drift apart.
    pub async fn delete(&self, key: &str) -> Result<(), ObjectStoreError> {
        let location = ObjectPath::from(key);
        self.bucket
            .delete(&location)
            .await
            .map_err(|source| request_failure(StoreOperation::Delete, key, source))?;

        Ok(())
    }
}
