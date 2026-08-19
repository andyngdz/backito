//! Writes the archive and proves it is not truncated before anything is sent.

use std::path::Path;
use std::sync::Arc;

use super::super::BackupError;
use crate::features::progress::{ProgressObserver, Step, human_bytes};
use crate::infra::docker::{PostgresTarget, count_table_data_entries, dump_to_file};

/// Fewest tables-with-data an archive must list to count as complete. One table
/// is a plausible small database; zero always means the dump produced nothing.
const MIN_TABLE_DATA_ENTRIES: usize = 1;

/// What a completed dump produced.
#[derive(Debug)]
pub struct ProducedArchive {
    /// Size of the archive on disk.
    pub bytes: u64,
    /// Tables the archive carries data for.
    pub tables: usize,
}

/// Dumps `target` to `archive_path`, then reads the archive back to confirm it
/// lists tables. A dump cut short by a full disk or a killed container fails
/// here, before it can be uploaded and mistaken for a good backup.
pub async fn produce_archive(
    target: &PostgresTarget,
    inspect_image: &str,
    archive_path: &Path,
    observer: &Arc<dyn ProgressObserver>,
) -> Result<ProducedArchive, BackupError> {
    observer.step_started(Step::Dump);
    dump_to_file(target, archive_path).await?;
    let bytes = file_size(archive_path)?;
    observer.step_finished(Step::Dump, &human_bytes(bytes));

    observer.step_started(Step::InspectArchive);
    let tables = inspect_archive(inspect_image, archive_path).await?;
    observer.step_finished(Step::InspectArchive, &format!("{tables} tables with data"));

    Ok(ProducedArchive { bytes, tables })
}

/// Counts the archive's tables in a throwaway container, never in the source
/// container: a live database's filesystem is not a scratch pad.
async fn inspect_archive(inspect_image: &str, archive_path: &Path) -> Result<usize, BackupError> {
    let tables = count_table_data_entries(inspect_image, archive_path).await?;

    if tables < MIN_TABLE_DATA_ENTRIES {
        return Err(BackupError::TooFewTables {
            found: tables,
            expected: MIN_TABLE_DATA_ENTRIES,
        });
    }
    Ok(tables)
}

/// Reads a local file's size.
pub(super) fn file_size(path: &Path) -> Result<u64, BackupError> {
    std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|source| BackupError::LocalFile {
            operation: "measure".to_owned(),
            path: path.to_string_lossy().into_owned(),
            source,
        })
}

#[cfg(test)]
#[path = "produce_archive_test.rs"]
mod produce_archive_test;
