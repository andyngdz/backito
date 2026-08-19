use super::{Step, human_bytes};

#[test]
fn every_step_reads_as_work_in_progress() {
    let steps = [
        Step::CheckDatabase,
        Step::CheckStorage,
        Step::Dump,
        Step::InspectArchive,
        Step::Checksum,
        Step::Upload,
        Step::Download,
        Step::StartScratch,
        Step::Restore,
        Step::CompareRows,
        Step::Cleanup,
    ];

    for step in steps {
        let label = step.label();
        assert!(!label.is_empty(), "{step:?} has no label");
        // A spinner line reads "Backing up database", not "Backup database" --
        // the participle is what makes it read as happening now.
        assert!(
            label
                .split(' ')
                .next()
                .is_some_and(|word| word.ends_with("ing")),
            "{step:?} label must start with a present participle, got {label:?}"
        );
    }
}

#[test]
fn a_step_displays_as_its_label() {
    assert_eq!(Step::Dump.to_string(), "Backing up database");
}

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
