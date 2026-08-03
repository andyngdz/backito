//! Hashes an archive without holding it in memory.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

use super::super::BackupError;
use crate::domain::ArchiveDigest;
use crate::features::progress::ProgressObserver;

/// Read buffer size. Large enough that syscall overhead disappears, small
/// enough that memory stays flat for any archive size.
const HASH_BUFFER_BYTES: usize = 1024 * 1024;

/// Computes the SHA-256 of the file at `path`, reporting bytes as it reads.
pub async fn digest_file(
    path: &Path,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<ArchiveDigest, BackupError> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|source| read_failure(path, source))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_BYTES];

    loop {
        let read = file
            .read(&mut buffer)
            .await
            .map_err(|source| read_failure(path, source))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        observer.transfer_advanced(read as u64);
    }

    Ok(ArchiveDigest::from_hex(hex::encode(hasher.finalize())))
}

/// Builds the failure for a file that could not be read.
fn read_failure(path: &Path, source: std::io::Error) -> BackupError {
    BackupError::LocalFile {
        operation: "hash".to_owned(),
        path: path.to_string_lossy().into_owned(),
        source,
    }
}

#[cfg(test)]
#[path = "checksum_test.rs"]
mod checksum_test;
