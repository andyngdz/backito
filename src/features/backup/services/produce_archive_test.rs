use super::file_size;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn a_written_file_reports_its_byte_count() {
    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(&vec![0_u8; 4096]).expect("write");
    file.flush().expect("flush");

    assert_eq!(file_size(file.path()).expect("size"), 4096);
}

#[test]
fn a_missing_file_fails_rather_than_reporting_zero() {
    // Reporting zero here would let an absent archive look like an empty one.
    let failure =
        file_size(std::path::Path::new("/nonexistent/archive.dump")).expect_err("must fail");

    assert!(failure.to_string().contains("measure"));
}
