//! The steps a command reports, in the order a user sees them.

use std::fmt;

use indicatif::HumanBytes;

/// One named stage of work. Every command reports the same vocabulary, so the
/// output shape does not change between `backup`, `verify`, and `restore`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Confirming the source container is up.
    CheckDatabase,
    /// Confirming the bucket answers with the configured credentials.
    CheckStorage,
    /// Writing the archive from the source database.
    Dump,
    /// Reading the archive back to confirm it is complete.
    InspectArchive,
    /// Hashing the archive.
    Checksum,
    /// Sending the archive to the bucket.
    Upload,
    /// Fetching an archive from the bucket.
    Download,
    /// Starting the throwaway database.
    StartScratch,
    /// Loading the archive into a database.
    Restore,
    /// Counting rows on both sides.
    CompareRows,
    /// Removing the throwaway database.
    Cleanup,
}

impl Step {
    /// The present-participle label shown while the step runs.
    pub fn label(self) -> &'static str {
        match self {
            Self::CheckDatabase => "Checking database connection",
            Self::CheckStorage => "Checking storage connection",
            Self::Dump => "Backing up database",
            Self::InspectArchive => "Inspecting archive",
            Self::Checksum => "Computing checksum",
            Self::Upload => "Uploading archive",
            Self::Download => "Downloading archive",
            Self::StartScratch => "Starting scratch database",
            Self::Restore => "Restoring archive",
            Self::CompareRows => "Comparing row counts",
            Self::Cleanup => "Removing scratch database",
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// Formats `bytes` in binary units, e.g. `882.34 MiB`.
///
/// Every size a user sees comes through here. Two formatters were in use before,
/// one per layer, which meant the same number could be printed two ways in a
/// single run depending on which step reported it.
pub fn human_bytes(bytes: u64) -> String {
    HumanBytes(bytes).to_string()
}

#[cfg(test)]
#[path = "domain_test.rs"]
mod domain_test;
