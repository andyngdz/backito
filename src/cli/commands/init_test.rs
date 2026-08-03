use super::run;
use crate::cli::ExitStatus;
use crate::features::init::Overwrite;

#[test]
fn init_reports_success_and_says_what_to_do_next() {
    // `init` runs against the working directory, so this exercises the same
    // path a user gets. The repository already carries both files, which is
    // exactly the case that must not be reported as a change.
    let report = run(Overwrite::Allow).expect("init");

    assert_eq!(report.status, ExitStatus::Success);
    let joined = report.lines.join("\n");
    assert!(joined.contains("backito.toml"));
    assert!(joined.contains("backito backup"));
}
