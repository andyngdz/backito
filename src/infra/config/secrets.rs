//! The secret half of a configuration: object-store credentials.
//!
//! Kept apart from the config sources on purpose. A `ConfigSource` reads a file
//! or the environment for the non-secret settings, and a `SecretSource` reads
//! the credentials, so a config file that leaks can never carry a token. The two
//! are joined only when [`super::Settings`] is assembled.

use std::fmt;

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
#[derive(Clone, PartialEq, Eq)]
pub struct StorageCredentials {
    /// S3 access key id.
    pub access_key_id: String,
    /// S3 secret access key.
    pub secret_access_key: String,
}

impl fmt::Debug for StorageCredentials {
    /// Redacts the secret.
    ///
    /// Written by hand rather than derived because `Settings` derives `Debug`
    /// and reaches this: one `tracing` field or one `{:?}` added later would
    /// otherwise put the token in a log that gets shipped somewhere.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        redact(formatter, "StorageCredentials", &self.access_key_id)
    }
}

/// Credentials for WAL storage.
///
/// Separate from the archive store's on purpose. A token scoped to one bucket
/// cannot damage the other if it leaks, and the whole point of a separate WAL
/// bucket is that it is separate.
#[derive(Clone, PartialEq, Eq)]
pub struct WalgCredentials {
    /// S3 access key id.
    pub access_key_id: String,
    /// S3 secret access key.
    pub secret_access_key: String,
}

impl fmt::Debug for WalgCredentials {
    /// Redacts the secret, for the same reason `StorageCredentials` does.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        redact(formatter, "WalgCredentials", &self.access_key_id)
    }
}

/// Renders a credential with its key id shown and its secret withheld.
///
/// The key id stays because it is what tells two credentials apart when one of
/// them is the wrong one, which is the thing a debug line is being read for.
fn redact(formatter: &mut fmt::Formatter<'_>, kind: &str, access_key_id: &str) -> fmt::Result {
    formatter
        .debug_struct(kind)
        .field("access_key_id", &access_key_id)
        .field("secret_access_key", &"<redacted>")
        .finish()
}

#[cfg(test)]
#[path = "secrets_test.rs"]
mod secrets_test;
