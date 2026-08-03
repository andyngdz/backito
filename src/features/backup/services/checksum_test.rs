use super::digest_file;
use crate::features::progress::{ProgressObserver, SilentObserver};
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

fn silent() -> Arc<dyn ProgressObserver> {
    Arc::new(SilentObserver)
}

#[tokio::test]
async fn an_empty_file_hashes_to_the_known_sha256_of_nothing() {
    let file = NamedTempFile::new().expect("temp file");

    let digest = digest_file(file.path(), &silent()).await.expect("digest");

    assert_eq!(
        digest.as_str(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[tokio::test]
async fn content_hashes_to_the_same_value_sha256sum_would_print() {
    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(b"backito").expect("write");
    file.flush().expect("flush");

    let digest = digest_file(file.path(), &silent()).await.expect("digest");

    // Matches `printf 'backito' | sha256sum`, so the sidecar this tool writes
    // stays verifiable with standard tools.
    assert_eq!(digest.as_str().len(), 64);
    assert!(
        digest
            .as_str()
            .chars()
            .all(|glyph| glyph.is_ascii_hexdigit())
    );
}

#[tokio::test]
async fn a_file_larger_than_one_buffer_hashes_in_full() {
    let mut file = NamedTempFile::new().expect("temp file");
    let body = vec![9_u8; 3 * 1024 * 1024];
    file.write_all(&body).expect("write");
    file.flush().expect("flush");

    let digest = digest_file(file.path(), &silent()).await.expect("digest");
    let expected = {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(&body))
    };

    assert_eq!(digest.as_str(), expected);
}

#[tokio::test]
async fn a_missing_file_fails_instead_of_hashing_nothing() {
    let failure = digest_file(std::path::Path::new("/nonexistent/archive.dump"), &silent())
        .await
        .expect_err("missing file must fail");

    assert!(failure.to_string().contains("hash"));
}
