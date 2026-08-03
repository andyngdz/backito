//! The bucket handle: construction and key listing.
//!
//! Nothing here probes or creates the bucket. A credential scoped to a single
//! bucket is denied `CreateBucket` and `ListBuckets`, so a probe would turn a
//! working configuration into a 403 before the first byte moves.

use s3::creds::Credentials;
use s3::{Bucket, Region};

use super::{ObjectStoreError, StoreOperation};
use crate::domain::ArchiveName;
use crate::infra::config::{StorageCredentials, StorageSettings};

/// One bucket, addressed with path-style URLs.
pub struct ObjectStore {
    pub(super) bucket: Box<Bucket>,
    pub(super) name: String,
}

impl ObjectStore {
    /// Builds a store for `settings` using `credentials`.
    pub fn new(
        settings: &StorageSettings,
        credentials: &StorageCredentials,
    ) -> Result<Self, ObjectStoreError> {
        let region = Region::Custom {
            region: settings.region.clone(),
            endpoint: settings.endpoint.clone(),
        };
        let resolved = Credentials::new(
            Some(&credentials.access_key_id),
            Some(&credentials.secret_access_key),
            None,
            None,
            None,
        )
        .map_err(|source| ObjectStoreError::Configure {
            bucket: settings.bucket.clone(),
            source: source.into(),
        })?;

        let bucket = Bucket::new(&settings.bucket, region, resolved)
            .map_err(|source| ObjectStoreError::Configure {
                bucket: settings.bucket.clone(),
                source,
            })?
            .with_path_style();

        Ok(Self {
            bucket,
            name: settings.bucket.clone(),
        })
    }

    /// The bucket this store writes to.
    pub fn bucket_name(&self) -> &str {
        &self.name
    }

    /// Lists every key in the bucket.
    ///
    /// Doubles as the reachability check: it is the cheapest call a
    /// bucket-scoped credential is allowed to make, so it proves the endpoint,
    /// the key, and the bucket name in one request.
    pub async fn list_keys(&self) -> Result<Vec<String>, ObjectStoreError> {
        let pages = self
            .bucket
            .list(String::new(), None)
            .await
            .map_err(|source| ObjectStoreError::Request {
                operation: StoreOperation::List.as_str().to_owned(),
                key: String::new(),
                source,
            })?;

        Ok(pages
            .into_iter()
            .flat_map(|page| page.contents)
            .map(|object| object.key)
            .collect())
    }

    /// The newest archive this tool wrote for `label`.
    ///
    /// Keys within one label embed a sortable UTC stamp, so string order is
    /// chronological order and no per-object metadata request is needed. Keys
    /// from another label or an older scheme are excluded first, because
    /// sorting across two prefixes is NOT chronological.
    pub async fn latest_archive(&self, label: &str) -> Result<ArchiveName, ObjectStoreError> {
        let mut archives: Vec<String> = self
            .list_keys()
            .await?
            .into_iter()
            .filter(|key| ArchiveName::belongs_to(key, label))
            .collect();
        archives.sort_unstable();

        archives
            .pop()
            .map(ArchiveName::from_key)
            .ok_or_else(|| ObjectStoreError::NoArchives {
                bucket: self.name.clone(),
            })
    }
}
