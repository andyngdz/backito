//! A per-run scratch directory that removes itself, and a sweep that reclaims
//! the ones a hard-killed run left behind.
//!
//! `TempDir` frees the directory on a clean exit or a `?`, but nothing runs when
//! the process is killed (OOM, a full disk, SIGKILL), so a multi-GB dump can
//! outlive the run. Each run holds an exclusive lock on its own workspace for
//! its whole life; the OS drops that lock the instant the process dies, however
//! it dies. A later run then tells a dead workspace (lock free) from a live one
//! (lock held) with certainty, with no guess at directory ages.

use std::fs::File;
use std::path::Path;

use tempfile::{Builder, TempDir};

/// Prefix shared by every backito scratch directory. Each command adds its own
/// suffix so a stray dir is recognisable, and the sweep keys off this common
/// stem so any command's start reclaims any other's abandoned scratch.
const SCRATCH_PREFIX: &str = "backito-";

/// A scratch directory owned for the length of one run.
///
/// Holds an exclusive lock on the directory so a concurrent sweep leaves it
/// alone, and removes the directory when dropped. The lock field is declared
/// before the directory so the lock is released before the directory is removed.
pub struct Workspace {
    _lock: File,
    directory: TempDir,
}

impl Workspace {
    /// Reclaims dead scratch dirs, then opens a fresh locked one under `prefix`.
    ///
    /// `prefix` names the owning command, e.g. `backito-verify-`. The reclaim
    /// runs on every acquire so any command's start clears leftovers, and it can
    /// never remove a live run's dir because that run holds its lock.
    pub fn acquire(prefix: &str) -> std::io::Result<Self> {
        sweep_stale(&std::env::temp_dir());

        let directory = Builder::new().prefix(prefix).tempdir()?;
        let lock = File::open(directory.path())?;
        // A just-created unique directory can only be locked by us, so a refusal
        // here is a real filesystem fault rather than contention.
        lock.try_lock().map_err(|failure| match failure {
            std::fs::TryLockError::Error(source) => source,
            std::fs::TryLockError::WouldBlock => std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "a fresh workspace was already locked",
            ),
        })?;

        Ok(Self {
            _lock: lock,
            directory,
        })
    }

    /// Path of the scratch directory.
    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}

/// Removes every scratch directory under `root` whose owning run has exited.
///
/// An unreadable root, or an entry that will not lock or delete, is skipped
/// rather than failing a startup: a leftover dir is recoverable, a daemon that
/// refuses to start is not.
fn sweep_stale(root: &Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with(SCRATCH_PREFIX) {
            continue;
        }
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            reclaim_if_dead(&entry.path());
        }
    }
}

/// Removes `directory` only when no live run holds its lock.
///
/// A lock that can be taken means the owner has exited: cleanly, in which case
/// `TempDir` already removed the dir, or by a kill, in which case the OS freed
/// the lock and this reclaims the leftover. A lock that is held belongs to a
/// running command and is left untouched.
fn reclaim_if_dead(directory: &Path) {
    let Ok(handle) = File::open(directory) else {
        return;
    };
    match handle.try_lock() {
        Ok(()) => {
            if let Err(failure) = std::fs::remove_dir_all(directory) {
                tracing::warn!(
                    path = %directory.display(),
                    %failure,
                    "could not remove a dead workspace, leaving it for the operator"
                );
            }
        }
        // A live run owns it; sparing it is the whole point of the lock.
        Err(std::fs::TryLockError::WouldBlock) => {}
        Err(std::fs::TryLockError::Error(failure)) => tracing::warn!(
            path = %directory.display(),
            %failure,
            "could not test a workspace lock, leaving the directory alone"
        ),
    }
}

#[cfg(test)]
#[path = "workspace_test.rs"]
mod workspace_test;
