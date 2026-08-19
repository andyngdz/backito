//! Sending bytes to the bucket.
//!
//! Uploads are multipart with an explicit ceiling on both the part size and the
//! number of parts in flight, so peak memory is a constant this file states
//! rather than something the host's free memory decides. That distinction is
//! the whole reason this tool does not use `put_object_stream` from `rust-s3`,
//! which sizes its own concurrency from `sysinfo` and cannot see a cgroup limit.

use std::path::Path;

use object_store::path::Path as ObjectPath;
use object_store::{ObjectStoreExt as _, PutPayload, WriteMultipart};
use tokio::io::{AsyncRead, AsyncReadExt};

use super::failures::{local_failure, read_failure, request_failure};
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

        if let Err(failure) = write_all_parts(key, &mut writer, reader).await {
            abort_quietly(key, writer).await;
            return Err(failure);
        }

        match writer.finish().await {
            Ok(_) => Ok(()),
            Err(source) => Err(request_failure(StoreOperation::Upload, key, source)),
        }
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
}

/// Streams `reader` into an open multipart upload, bounding parts in flight.
///
/// Split from its caller so every early return lands in one place, which is what
/// makes aborting the upload reliable rather than something each `?` has to
/// remember.
async fn write_all_parts<R: AsyncRead + Unpin + ?Sized>(
    key: &str,
    writer: &mut WriteMultipart,
    reader: &mut R,
) -> Result<(), ObjectStoreError> {
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

    Ok(())
}

/// Drops the parts already accepted for an upload that will not finish.
///
/// They outlive the process that sent them and are billed until a lifecycle
/// rule reaps them, so a daemon retrying a failing multi-GB upload every fifteen
/// minutes would otherwise pile them up indefinitely. The abort itself is
/// reported and not propagated: the upload has already failed, and replacing its
/// cause with a cleanup failure hides the reason the operator needs.
async fn abort_quietly(key: &str, writer: WriteMultipart) {
    if let Err(source) = writer.abort().await {
        tracing::warn!(
            key,
            %source,
            "could not abort a part-uploaded archive, so its parts stay billed until \
             the bucket lifecycle rule removes them"
        );
    }
}

#[cfg(test)]
#[path = "upload_test.rs"]
mod upload_test;
