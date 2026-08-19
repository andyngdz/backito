//! The per-pass scratch directory and the sweep that clears ones a killed
//! earlier run left behind.

use std::path::Path;

use tempfile::{Builder, TempDir};

/// Prefix every daemon pass workspace carries. The startup sweep keys off this,
/// so a standalone `backup`/`verify`/`restore` workspace (a different prefix) is
/// never mistaken for an abandoned daemon dir and removed out from under a live
/// run.
pub const PASS_PREFIX: &str = "backito-daemon-";

/// A fresh workspace for one pass, removed when the returned guard drops.
///
/// One backup pass stages a multi-GB dump in here. Reusing a single directory
/// for the daemon's whole life is what let those dumps pile up until the host
/// disk filled, so each pass gets its own and frees it on the way out.
pub fn pass_workspace() -> std::io::Result<TempDir> {
    Builder::new().prefix(PASS_PREFIX).tempdir()
}

/// Removes every pass workspace left under `root` by an earlier run, returning
/// how many it removed.
///
/// A fresh daemon has no pass in flight, so any `backito-daemon-*` directory is
/// abandoned scratch from a process killed before its guard could run. A dir
/// that will not delete is logged and skipped rather than stopping startup, and
/// an unreadable root is treated as nothing to sweep.
pub fn sweep_stale(root: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(root) else {
        return 0;
    };

    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with(PASS_PREFIX) {
            continue;
        }
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        match std::fs::remove_dir_all(entry.path()) {
            Ok(()) => removed += 1,
            Err(failure) => tracing::warn!(
                path = %entry.path().display(),
                %failure,
                "could not remove a stale workspace, leaving it for the operator"
            ),
        }
    }
    removed
}

#[cfg(test)]
#[path = "workspace_test.rs"]
mod workspace_test;
