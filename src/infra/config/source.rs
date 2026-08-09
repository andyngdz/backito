//! Where configuration comes from.
//!
//! Two traits, because a configuration has two halves with different rules. A
//! [`ConfigSource`] supplies the non-secret settings, read from exactly one
//! place: a file or the environment, never both filling each other's gaps. A
//! [`SecretSource`] supplies the credentials. Keeping them apart means a leaked
//! config source cannot carry a token, and a new backend (a secret manager, a
//! remote config service) is one more implementation rather than a rewrite.

use super::{ConfigCore, ConfigError, Secrets};

/// A complete set of non-secret settings, read from one place.
///
/// Implementations do not consult each other: whichever source is chosen owns
/// every field, and a missing one is that source's error rather than a silent
/// fall-through to another.
pub trait ConfigSource {
    /// Reads the settings, or explains why they could not be read.
    fn load(&self) -> Result<ConfigCore, ConfigError>;
}

/// The credentials a configuration needs, read from one place.
pub trait SecretSource {
    /// Reads the credentials, or explains why they could not be read.
    fn load(&self) -> Result<Secrets, ConfigError>;
}
