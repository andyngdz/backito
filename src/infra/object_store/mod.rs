//! S3-compatible object storage for one bucket.

mod errors;
mod operation;
mod store;
mod transfer;

pub use errors::ObjectStoreError;
pub use operation::StoreOperation;
pub use store::ObjectStore;
