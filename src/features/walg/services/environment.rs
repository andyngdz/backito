//! The environment `wal-g` reads its own configuration from.
//!
//! wal-g takes no config file and no flags for these: it reads `WALG_S3_PREFIX`
//! and the `AWS_*` set from its process environment. backito owns the values in
//! typed config and hands them over here, so there is one place that knows the
//! spelling wal-g expects.

use crate::infra::config::{WalgCredentials, WalgSettings};

/// The value S3-compatible boolean settings expect.
const ENABLED: &str = "true";

/// Name/value pairs to set on a `wal-g` process.
///
/// Returned rather than applied to this process: setting a variable here would
/// leak into every later command, and `std::env::set_var` is unsafe in a
/// threaded program for exactly that reason.
pub fn walg_environment(settings: &WalgSettings) -> Vec<(&'static str, String)> {
    vec![
        ("WALG_S3_PREFIX", settings.s3_prefix.clone()),
        ("AWS_ENDPOINT", settings.endpoint.clone()),
        ("AWS_REGION", settings.region.clone()),
        credential_pair_id(&settings.credentials),
        credential_pair_secret(&settings.credentials),
        // R2 and most S3-compatible services address buckets by path rather
        // than by subdomain. Without this wal-g builds virtual-host URLs that
        // resolve nowhere.
        ("AWS_S3_FORCE_PATH_STYLE", ENABLED.to_owned()),
    ]
}

/// The access key id, under the name wal-g reads.
fn credential_pair_id(credentials: &WalgCredentials) -> (&'static str, String) {
    ("AWS_ACCESS_KEY_ID", credentials.access_key_id.clone())
}

/// The secret access key, under the name wal-g reads.
fn credential_pair_secret(credentials: &WalgCredentials) -> (&'static str, String) {
    (
        "AWS_SECRET_ACCESS_KEY",
        credentials.secret_access_key.clone(),
    )
}

#[cfg(test)]
#[path = "environment_test.rs"]
mod environment_test;
