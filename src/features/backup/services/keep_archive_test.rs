use super::keep_archive;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn the_archive_lands_under_its_own_name() {
    let mut source = NamedTempFile::new().expect("temp file");
    source.write_all(b"archive bytes").expect("write");
    source.flush().expect("flush");
    let directory = TempDir::new().expect("temp dir");

    let kept = keep_archive(
        source.path(),
        directory.path(),
        "app-backup-20260803-0942.dump",
    )
    .expect("keep");

    assert_eq!(
        kept.file_name().and_then(|name| name.to_str()),
        Some("app-backup-20260803-0942.dump")
    );
    assert_eq!(std::fs::read(&kept).expect("read"), b"archive bytes");
}

#[test]
fn an_unwritable_destination_fails_with_the_path_it_tried() {
    let source = NamedTempFile::new().expect("temp file");

    let failure = keep_archive(
        source.path(),
        std::path::Path::new("/nonexistent-directory"),
        "archive.dump",
    )
    .expect_err("must fail");

    let message = failure.to_string();
    assert!(message.contains("/nonexistent-directory/archive.dump"));
    assert!(message.contains("keep"));
}
