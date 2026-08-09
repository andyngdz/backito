//! Reads credentials from the environment.
//!
//! The archive token is required. The WAL token is optional here: whether a
//! project needs it depends on its `[walg]` section, which lives in the config
//! source, not here. A half-set WAL token is an error rather than a silent
//! absence, so a typo in one of the two names fails loudly.

use super::{ConfigError, SecretSource, Secrets, StorageCredentials, WalgCredentials, env};

/// Environment variable holding the archive-store access key id.
const ACCESS_KEY_VAR: &str = "BACKITO_ACCESS_KEY_ID";

/// Environment variable holding the archive-store secret access key.
const SECRET_KEY_VAR: &str = "BACKITO_SECRET_ACCESS_KEY";

/// Environment variable holding the WAL-store access key id.
const WALG_ACCESS_KEY_VAR: &str = "BACKITO_WALG_ACCESS_KEY_ID";

/// Environment variable holding the WAL-store secret access key.
const WALG_SECRET_KEY_VAR: &str = "BACKITO_WALG_SECRET_ACCESS_KEY";

/// Reads credentials from `BACKITO_*` variables.
pub struct EnvSecretSource;

impl SecretSource for EnvSecretSource {
    fn load(&self) -> Result<Secrets, ConfigError> {
        let walg = match walg_credentials()? {
            WalgSecret::Present(credentials) => Some(credentials),
            WalgSecret::Absent => None,
        };
        Ok(Secrets {
            storage: StorageCredentials {
                access_key_id: env::required(ACCESS_KEY_VAR)?,
                secret_access_key: env::required(SECRET_KEY_VAR)?,
            },
            walg,
        })
    }
}

/// Whether the WAL token was set in the environment.
enum WalgSecret {
    /// Both names were present.
    Present(WalgCredentials),
    /// Neither name was set; a project without WAL archiving needs no token.
    Absent,
}

/// Reads the WAL token if it is set. Both names present gives the token; both
/// absent gives none; one present is a mistake worth naming.
fn walg_credentials() -> Result<WalgSecret, ConfigError> {
    match (
        env::optional(WALG_ACCESS_KEY_VAR),
        env::optional(WALG_SECRET_KEY_VAR),
    ) {
        (Some(access_key_id), Some(secret_access_key)) => {
            Ok(WalgSecret::Present(WalgCredentials {
                access_key_id,
                secret_access_key,
            }))
        }
        (None, None) => Ok(WalgSecret::Absent),
        (Some(_), None) => Err(ConfigError::MissingEnvVar {
            variable: WALG_SECRET_KEY_VAR.to_owned(),
        }),
        (None, Some(_)) => Err(ConfigError::MissingEnvVar {
            variable: WALG_ACCESS_KEY_VAR.to_owned(),
        }),
    }
}

#[cfg(test)]
#[path = "env_secret_source_test.rs"]
mod env_secret_source_test;
