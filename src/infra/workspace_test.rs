use std::fs::{self, File};
use std::path::PathBuf;

use tempfile::TempDir;

use super::{Workspace, sweep_stale};

/// Creates a scratch-style child dir of `root` holding one file, so a sweep has
/// real contents to remove rather than an empty directory.
fn scratch_child(root: &TempDir, name: &str) -> PathBuf {
    let path = root.path().join(name);
    fs::create_dir(&path).expect("create dir");
    fs::write(path.join("dump"), b"leftover").expect("write file");
    path
}

#[test]
fn sweep_removes_a_dead_workspace_and_spares_a_live_one() {
    let root = TempDir::new().expect("root");
    let dead = scratch_child(&root, "backito-verify-dead");
    let live = scratch_child(&root, "backito-daemon-live");
    let unrelated = scratch_child(&root, "unrelated-keep");

    // Hold the live dir's lock, standing in for a concurrent running command.
    let guard = File::open(&live).expect("open live");
    guard.try_lock().expect("lock live");

    sweep_stale(root.path());

    assert!(
        !dead.exists(),
        "a workspace with no live owner must be removed"
    );
    assert!(live.exists(), "a locked workspace must survive the sweep");
    assert!(unrelated.exists(), "a non-backito dir must be left alone");

    let _ = guard.unlock();
}

#[test]
fn sweep_of_an_unreadable_root_does_not_panic() {
    let root = TempDir::new().expect("root");
    sweep_stale(&root.path().join("does-not-exist"));
}

#[test]
fn a_failed_run_still_frees_its_workspace() {
    // A run that fails partway drops its Workspace on the way out, and the drop
    // removes the directory whether the run returned Ok or Err. A pass that
    // failed mid-dump is exactly what left the multi-GB dirs behind.
    let leaked;
    {
        let workspace = Workspace::acquire("backito-test-").expect("acquire");
        leaked = workspace.path().to_owned();
        assert!(leaked.exists());

        let run: Result<(), &str> = Err("pg_dump killed, disk full");
        assert!(run.is_err());
        // workspace drops here, exactly as it does after a failed run
    }

    assert!(
        !leaked.exists(),
        "workspace must be gone once the run returns, success or failure"
    );
}
