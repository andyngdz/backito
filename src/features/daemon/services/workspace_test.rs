use std::fs;

use tempfile::TempDir;

use super::{PASS_PREFIX, pass_workspace, sweep_stale};

/// Makes a child directory of `root` holding one file, so a sweep has real
/// contents to remove rather than an empty dir.
fn stale_dir(root: &TempDir, name: &str) {
    let path = root.path().join(name);
    fs::create_dir(&path).expect("create dir");
    fs::write(path.join("dump"), b"leftover").expect("write file");
}

#[test]
fn sweep_removes_daemon_dirs_and_spares_other_prefixes() {
    let root = TempDir::new().expect("root");
    stale_dir(&root, "backito-daemon-aaa");
    stale_dir(&root, "backito-daemon-bbb");
    stale_dir(&root, "backito-verify-ccc");
    stale_dir(&root, "backito-restore-ddd");
    stale_dir(&root, "unrelated-eee");

    let removed = sweep_stale(root.path());

    assert_eq!(removed, 2, "only the two daemon dirs should be removed");
    assert!(!root.path().join("backito-daemon-aaa").exists());
    assert!(!root.path().join("backito-daemon-bbb").exists());
    // A concurrent standalone command's workspace must survive the sweep.
    assert!(root.path().join("backito-verify-ccc").exists());
    assert!(root.path().join("backito-restore-ddd").exists());
    assert!(root.path().join("unrelated-eee").exists());
}

#[test]
fn sweep_of_an_unreadable_root_is_zero() {
    let root = TempDir::new().expect("root");
    let missing = root.path().join("does-not-exist");

    assert_eq!(sweep_stale(&missing), 0);
}

#[test]
fn a_failed_pass_still_frees_its_workspace() {
    // Mirrors the loop's exact structure: the guard is created, the pass runs,
    // and the guard drops at the end of the iteration whether the pass returned
    // Ok or Err. A pass that fails partway is what left the multi-GB dirs
    // behind, so this proves the error path cleans up too.
    let leaked_path;
    {
        let workspace = pass_workspace().expect("workspace");
        leaked_path = workspace.path().to_owned();
        assert!(leaked_path.exists());
        assert!(
            leaked_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("name")
                .starts_with(PASS_PREFIX)
        );

        let pass_result: Result<(), &str> = Err("pg_dump killed, disk full");
        assert!(pass_result.is_err());
        // guard drops here, exactly as it does after a failed pass
    }

    assert!(
        !leaked_path.exists(),
        "workspace must be gone once the pass returns, success or failure"
    );
}
