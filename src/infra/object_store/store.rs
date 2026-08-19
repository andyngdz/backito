//! The bucket handle: construction and key listing.
//!
//! Nothing here probes or creates the bucket. A credential scoped to a single
//! bucket is denied `CreateBucket` and `ListBuckets`, so a probe would turn a
//! working configuration into a 403 before the first byte moves.

use futures::TryStreamExt;
// `list` sits on the trait itself; the convenience wrappers this crate uses
// elsewhere are on ObjectStoreExt. Imported anonymously so it cannot collide
// with this module's own ObjectStore.
use object_store::ObjectMeta;
use object_store::ObjectStore as _;
use object_store::aws::{AmazonS3, AmazonS3Builder};

use super::{ObjectStoreError, StoreOperation};
use crate::domain::{ArchiveName, StoredArchive};
use crate::infra::config::{StorageCredentials, StorageSettings};

/// One bucket, addressed with path-style URLs.
pub struct ObjectStore {
    pub(super) bucket: AmazonS3,
    pub(super) name: String,
}

impl ObjectStore {
    /// Builds a store for `settings` using `credentials`.
    pub fn new(
        settings: &StorageSettings,
        credentials: &StorageCredentials,
    ) -> Result<Self, ObjectStoreError> {
        // Virtual-hosted style is off because an S3-compatible endpoint is
        // addressed as `<endpoint>/<bucket>/<key>`, not as a subdomain.
        let bucket = AmazonS3Builder::new()
            .with_endpoint(settings.endpoint.clone())
            .with_bucket_name(settings.bucket.clone())
            .with_region(settings.region.clone())
            .with_access_key_id(credentials.access_key_id.clone())
            .with_secret_access_key(credentials.secret_access_key.clone())
            .with_virtual_hosted_style_request(false)
            .build()
            .map_err(|source| ObjectStoreError::Configure {
                bucket: settings.bucket.clone(),
                source: Box::new(source),
            })?;

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
        Ok(self
            .list_objects()
            .await?
            .into_iter()
            .map(|object| object.location.to_string())
            .collect())
    }

    /// Every archive this tool wrote for `label`, oldest first.
    ///
    /// Sizes come from the listing rather than a `head` per object, so asking
    /// what a bucket holds costs the same one request whether it holds three
    /// archives or three hundred.
    pub async fn list_archives(&self, label: &str) -> Result<Vec<StoredArchive>, ObjectStoreError> {
        let mut archives: Vec<StoredArchive> = self
            .list_objects()
            .await?
            .into_iter()
            .filter(|object| ArchiveName::belongs_to(object.location.as_ref(), label))
            .map(|object| StoredArchive {
                name: ArchiveName::from_key(object.location.to_string()),
                bytes: object.size,
            })
            .collect();

        // Within one label the stamp sorts chronologically, so key order is age
        // order and no metadata request is needed to establish it.
        archives.sort_unstable_by(|left, right| left.name.cmp(&right.name));
        Ok(archives)
    }

    /// The newest archive this tool wrote for `label`.
    ///
    /// Keys within one label embed a sortable UTC stamp, so string order is
    /// chronological order and no per-object metadata request is needed. Keys
    /// from another label or an older scheme are excluded first, because
    /// sorting across two prefixes is NOT chronological.
    pub async fn latest_archive(&self, label: &str) -> Result<ArchiveName, ObjectStoreError> {
        self.list_archives(label)
            .await?
            .pop()
            .map(|archive| archive.name)
            .ok_or_else(|| ObjectStoreError::NoArchives {
                bucket: self.name.clone(),
            })
    }

    /// Lists the bucket's objects with the metadata the listing already carries.
    async fn list_objects(&self) -> Result<Vec<ObjectMeta>, ObjectStoreError> {
        self.bucket
            .list(None)
            .try_collect()
            .await
            .map_err(|source| ObjectStoreError::Request {
                operation: StoreOperation::List.as_str().to_owned(),
                key: String::new(),
                source: Box::new(source),
            })
    }
}
