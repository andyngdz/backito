use super::CommandReport;
use crate::cli::ExitStatus;

#[test]
fn a_single_line_result_succeeds() {
    let report = CommandReport::line("app-backup-20260803-0942.dump");

    assert_eq!(report.lines, vec!["app-backup-20260803-0942.dump"]);
    assert_eq!(report.status, ExitStatus::Success);
}

#[test]
fn a_report_can_carry_a_mismatch_status_with_its_lines() {
    // verify prints its findings and still exits non-zero.
    let report = CommandReport::lines(vec!["FAIL  ...".to_owned()], ExitStatus::Mismatch);

    assert_eq!(report.status, ExitStatus::Mismatch);
    assert_eq!(report.lines.len(), 1);
}
