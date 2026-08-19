//! S3-compatible object storage for one bucket.

mod download;
mod errors;
mod failures;
mod operation;
mod store;
mod upload;

pub use errors::ObjectStoreError;
pub use operation::StoreOperation;
pub use store::{ObjectStore, StoredArchive};
