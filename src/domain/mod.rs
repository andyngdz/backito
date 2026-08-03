//! Values every command shares: what an archive is called, and how a restored
//! database is compared to its source.

mod archive;
mod table_count;

pub use archive::{ArchiveDigest, ArchiveName};
pub use table_count::{CountVerdict, TableComparison, TableCounts, compare_counts, rows_behind};
