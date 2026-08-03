use super::RestoreOutcome;
use crate::domain::ArchiveName;

#[test]
fn an_outcome_keeps_the_archive_it_restored() {
    let outcome = RestoreOutcome {
        archive: ArchiveName::new("app", "20260803-0942"),
        bytes: 925_177_935,
        restore_stderr: String::new(),
    };

    // The key is what a user needs to reproduce or audit the restore.
    assert_eq!(outcome.archive.as_str(), "app-backup-20260803-0942.dump");
}
