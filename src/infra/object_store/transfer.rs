//! Moving bytes in and out of the bucket.
//!
//! Uploads are multipart with an explicit ceiling on both the part size and the
//! number of parts in flight, so peak memory is a constant this file states
//! rather than something the host's free memory decides. That distinction is
//! the whole reason this tool does not use `put_object_stream` from `rust-s3`,
//! which sizes its own concurrency from `sysinfo` and cannot see a cgroup limit.

use std::path::Path;

use futures::StreamExt;
use object_store::path::Path as ObjectPath;
use object_store::{ObjectStoreExt as _, PutPayload, WriteMultipart};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{ObjectStore, ObjectStoreError, StoreOperation};

/// Bytes per multipart part.
const PART_BYTES: usize = 8 * 1024 * 1024;

/// Parts allowed in flight at once. One would serialise the upload, sending a
/// part and then idling the link while the next is read.
const MAX_PARTS_IN_FLIGHT: usize = 4;

/// Smallest part S3 accepts, for every part but the last.
const S3_MINIMUM_PART_BYTES: usize = 5 * 1024 * 1024;

/// What a container running this tool has to be allowed to use for an upload.
const PEAK_UPLOAD_BUDGET_BYTES: usize = 64 * 1024 * 1024;

// Checked here rather than in a test because both are facts about the constants
// above, and a too-small part only fails at runtime on the second part, which no
// small-file test would ever reach.
const _: () = assert!(PART_BYTES >= S3_MINIMUM_PART_BYTES);
const _: () = assert!(
    // The parts in flight, the part being filled, and the read buffer below.
    PART_BYTES * (MAX_PARTS_IN_FLIGHT + 2) <= PEAK_UPLOAD_BUDGET_BYTES
);

impl ObjectStore {
    /// Uploads `reader` to `key` as a multipart stream.
    pub async fn upload_stream<R: AsyncRead + Unpin + ?Sized>(
        &self,
        key: &str,
        reader: &mut R,
    ) -> Result<(), ObjectStoreError> {
        let location = ObjectPath::from(key);
        let upload = self
            .bucket
            .put_multipart(&location)
            .await
            .map_err(|source| request_failure(StoreOperation::Upload, key, source))?;

        let mut writer = WriteMultipart::new_with_chunk_size(upload, PART_BYTES);
        let mut read_buffer = vec![0_u8; PART_BYTES];

        loop {
            let filled = reader
                .read(&mut read_buffer)
                .await
                .map_err(|source| read_failure(key, source))?;
            if filled == 0 {
                break;
            }

            // Before handing over more bytes, block until the upload has drained
            // enough parts. `write` alone starts a new part the moment the chunk
            // fills, however many are already in flight.
            writer
                .wait_for_capacity(MAX_PARTS_IN_FLIGHT)
                .await
                .map_err(|source| request_failure(StoreOperation::Upload, key, source))?;
            writer.write(&read_buffer[..filled]);
        }

        writer
            .finish()
            .await
            .map_err(|source| request_failure(StoreOperation::Upload, key, source))?;

        Ok(())
    }

    /// Uploads a small in-memory body, used for the checksum sidecar.
    pub async fn upload_bytes(&self, key: &str, body: &[u8]) -> Result<(), ObjectStoreError> {
        let location = ObjectPath::from(key);
        self.bucket
            .put(&location, PutPayload::from(body.to_vec()))
            .await
            .map_err(|source| request_failure(StoreOperation::Upload, key, source))?;

        Ok(())
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
}

/// Builds the failure for a request that never got a usable answer.
fn request_failure(
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
fn read_failure(key: &str, source: std::io::Error) -> ObjectStoreError {
    ObjectStoreError::LocalStream {
        operation: StoreOperation::Upload.as_str().to_owned(),
        key: key.to_owned(),
        source,
    }
}

/// Builds the failure for writing the local side of a download.
fn write_failure(key: &str, source: std::io::Error) -> ObjectStoreError {
    ObjectStoreError::LocalStream {
        operation: StoreOperation::Download.as_str().to_owned(),
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
