//! Retention: keep the newest `retain` archives for a label, delete the rest.

use crate::domain::ArchiveName;
use crate::infra::object_store::{ObjectStore, ObjectStoreError};

/// What a prune pass did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneOutcome {
    /// Archives kept, including the one just written.
    pub kept: usize,
    /// Keys deleted, each with its checksum sidecar.
    pub deleted: Vec<String>,
}

/// Deletes every archive for `label` older than the newest `retain`.
///
/// Only keys matching `<label>-backup-<stamp>.dump` are considered, so a bucket
/// shared with another label, or with objects this tool did not write, loses
/// nothing. Within one label the stamp sorts chronologically, so string order is
/// age order and no metadata request is needed.
pub async fn prune_archives(
    store: &ObjectStore,
    label: &str,
    retain: u32,
) -> Result<PruneOutcome, ObjectStoreError> {
    let stored = store.list_keys().await?;
    let expiring = archives_to_drop(&stored, label, retain);
    if expiring.is_empty() {
        return Ok(PruneOutcome {
            kept: stored
                .iter()
                .filter(|key| ArchiveName::belongs_to(key, label))
                .count(),
            deleted: Vec::new(),
        });
    }

    let mut deleted = Vec::with_capacity(expiring.len());
    for key in expiring {
        store.delete(&key).await?;
        // The sidecar is best-effort: an archive whose checksum is already gone
        // still has to lose its dump, and failing the pass here would leave the
        // bucket growing forever over one orphaned file.
        let checksum_key = ArchiveName::from_key(key.clone()).checksum_key();
        let _ = store.delete(&checksum_key).await;
        deleted.push(key);
    }

    Ok(PruneOutcome {
        kept: retain as usize,
        deleted,
    })
}

/// The keys to delete: every archive for `label` older than the newest
/// `retain`.
///
/// Split out of the deleting so the rule can be read and tested on its own.
/// Only keys matching `<label>-backup-<stamp>.dump` are candidates, so a bucket
/// shared with another label, or holding objects this tool did not write, loses
/// nothing. Within one label the stamp sorts chronologically, so string order is
/// age order and no metadata request is needed.
pub fn archives_to_drop(stored: &[String], label: &str, retain: u32) -> Vec<String> {
    let mut archives: Vec<&String> = stored
        .iter()
        .filter(|key| ArchiveName::belongs_to(key, label))
        .collect();
    archives.sort_unstable();

    let retain_count = retain as usize;
    let Some(drop_count) = archives.len().checked_sub(retain_count) else {
        return Vec::new();
    };

    archives.into_iter().take(drop_count).cloned().collect()
}

#[cfg(test)]
#[path = "prune_test.rs"]
mod prune_test;
