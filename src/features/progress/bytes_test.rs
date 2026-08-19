use super::human_bytes;

#[test]
fn sizes_are_rendered_in_binary_units() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(1024), "1.00 KiB");
    assert_eq!(human_bytes(1024 * 1024), "1.00 MiB");
}

#[test]
fn a_multi_gigabyte_archive_stays_readable() {
    // The sizes this tool reports are dump-sized, so the large end is the end
    // that matters.
    assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.00 GiB");
}
