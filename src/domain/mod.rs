//! Values every command shares: what an archive is called, and how a restored
//! database is compared to its source.

mod archive;
mod errors;
mod interval;
mod table_count;

pub use archive::{ArchiveDigest, ArchiveName, stamp_at, stamp_taken_at};
pub use errors::IntervalError;
pub use interval::Interval;
pub use table_count::{CountVerdict, TableComparison, TableCounts, compare_counts, rows_behind};
