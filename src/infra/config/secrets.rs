//! The secret half of a configuration: object-store credentials.
//!
//! Kept apart from the config sources on purpose. A `ConfigSource` reads a file
//! or the environment for the non-secret settings, and a `SecretSource` reads
//! the credentials, so a config file that leaks can never carry a token. The two
//! are joined only when [`super::Settings`] is assembled.

/// Credentials for the object store, and for WAL storage when it is configured.
#[derive(Debug, Clone)]
pub struct Secrets {
    /// Credentials for the archive bucket.
    pub storage: StorageCredentials,
    /// Credentials for WAL storage, present only when a secret source found
    /// them. Whether they are required is decided when settings are assembled:
    /// a config with a `[walg]` section needs them, one without does not.
    pub walg: Option<WalgCredentials>,
}

/// Credentials for the archive bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageCredentials {
    /// S3 access key id.
    pub access_key_id: String,
    /// S3 secret access key.
    pub secret_access_key: String,
}

/// Credentials for WAL storage.
///
/// Separate from the archive store's on purpose. A token scoped to one bucket
/// cannot damage the other if it leaks, and the whole point of a separate WAL
/// bucket is that it is separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalgCredentials {
    /// S3 access key id.
    pub access_key_id: String,
    /// S3 secret access key.
    pub secret_access_key: String,
}
