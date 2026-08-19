//! The throwaway database a verification restores into.
//!
//! It carries a fixed name prefix, has no volume and no published port, and is
//! removed when the guard drops. Nothing outside this module should ever point
//! a restore at a database it did not create here.

use super::super::VerifyError;
use crate::infra::docker::{
    DOCKER_BIN, PostgresTarget, is_running, remove, start_throwaway, wait_ready,
};

/// Prefix every scratch container name carries, so one can always be told apart
/// from a real database by name alone.
pub const SCRATCH_PREFIX: &str = "backito-scratch-";

/// Database created inside the scratch container image.
const SCRATCH_DATABASE: &str = "postgres";

/// Role the scratch image creates.
const SCRATCH_USER: &str = "postgres";

/// A running scratch container, removed on drop.
pub struct ScratchDatabase {
    name: String,
}

impl ScratchDatabase {
    /// Starts a scratch container named for `label` from `image`.
    ///
    /// Refuses to reuse a container whose name lacks the scratch prefix, so a
    /// misconfigured run can never restore over a real database.
    pub async fn start(label: &str, image: &str) -> Result<Self, VerifyError> {
        let name = scratch_name(label);
        if !name.starts_with(SCRATCH_PREFIX) {
            return Err(VerifyError::ScratchNameTaken { container: name });
        }

        start_throwaway(&name, image).await?;
        wait_ready(&name, SCRATCH_USER).await?;
        Ok(Self { name })
    }

    /// The connection target for this scratch database.
    pub fn target(&self) -> PostgresTarget {
        PostgresTarget {
            container: self.name.clone(),
            database: SCRATCH_DATABASE.to_owned(),
            user: SCRATCH_USER.to_owned(),
        }
    }

    /// The container name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Removes the container. Called explicitly so failures are reported;
    /// `Drop` covers the paths that do not get here.
    pub async fn destroy(&self) -> Result<(), VerifyError> {
        remove(&self.name).await?;
        Ok(())
    }
}

impl Drop for ScratchDatabase {
    fn drop(&mut self) {
        // A blocking best-effort removal: the async path already ran on the
        // normal route, and this only covers panics and early returns. Leaving
        // a container behind is recoverable; blocking the runtime is not, so
        // this spawns a detached process rather than awaiting.
        let _ = std::process::Command::new(DOCKER_BIN)
            .args(["rm", "-f", &self.name])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Builds the scratch container name for `label`.
pub(super) fn scratch_name(label: &str) -> String {
    format!("{SCRATCH_PREFIX}{label}")
}

/// True when a scratch container from an earlier interrupted run is still up.
pub async fn leftover_exists(label: &str) -> Result<bool, VerifyError> {
    Ok(is_running(&scratch_name(label)).await?)
}

#[cfg(test)]
#[path = "scratch_test.rs"]
mod scratch_test;
