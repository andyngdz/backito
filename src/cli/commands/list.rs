//! `backito list`: report which archives the bucket holds.

use super::super::dto::CommandReport;
use super::super::{CliError, ExitStatus};
use crate::domain::{ArchiveName, stamp_taken_at};
use crate::features::progress::human_bytes;
use crate::features::verify::VerifyError;
use crate::infra::config::Settings;
use crate::infra::object_store::{ObjectStore, StoredArchive};

/// What the listing prints per archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// Key, size, and age, for a person reading the terminal.
    Full,
    /// Keys alone, for a pipe.
    KeysOnly,
}

impl From<bool> for Detail {
    /// Maps the `--keys-only` flag onto the shape it names.
    fn from(keys_only: bool) -> Self {
        match keys_only {
            true => Self::KeysOnly,
            false => Self::Full,
        }
    }
}

/// Lists the archives stored for the configured label.
pub async fn run(settings: &Settings, detail: Detail) -> Result<CommandReport, CliError> {
    let store =
        ObjectStore::new(&settings.storage, &settings.credentials).map_err(VerifyError::Storage)?;

    let archives = store
        .list_archives(&settings.database.label)
        .await
        .map_err(VerifyError::Storage)?;

    Ok(CommandReport::lines(
        render(&archives, detail),
        ExitStatus::Success,
    ))
}

/// Renders the listing.
///
/// An empty bucket is a successful answer, not a failure: "nothing here yet" is
/// exactly what someone running this after a fresh `init` needs to be told, and
/// exiting non-zero would make a shell think the listing itself broke.
fn render(archives: &[StoredArchive], detail: Detail) -> Vec<String> {
    if archives.is_empty() {
        return match detail {
            Detail::KeysOnly => Vec::new(),
            Detail::Full => vec!["no archives stored yet -- run `backito backup`".to_owned()],
        };
    }

    match detail {
        Detail::KeysOnly => archives
            .iter()
            .map(|archive| archive.name.to_string())
            .collect(),
        Detail::Full => {
            let widest = archives
                .iter()
                .map(|archive| archive.name.as_str().len())
                .max()
                .unwrap_or(0);
            archives
                .iter()
                .map(|archive| full_line(archive, widest))
                .collect()
        }
    }
}

/// One archive as a padded row: key, size, then when it was taken.
fn full_line(archive: &StoredArchive, key_width: usize) -> String {
    format!(
        "{:<key_width$}  {:>10}  {}",
        archive.name.as_str(),
        human_bytes(archive.bytes),
        taken_at(&archive.name)
    )
}

/// When the archive was taken, read out of its own key.
///
/// A key that reached the bucket without a readable stamp is shown rather than
/// hidden: it means something else wrote a key that passes the label filter, and
/// that is worth seeing in the listing.
fn taken_at(name: &ArchiveName) -> String {
    name.stamp().and_then(stamp_taken_at).map_or_else(
        || "unknown date".to_owned(),
        |taken| taken.strftime("%Y-%m-%d %H:%M UTC").to_string(),
    )
}

#[cfg(test)]
#[path = "list_test.rs"]
mod list_test;
