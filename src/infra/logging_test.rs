use super::LogDetail;

#[test]
fn normal_runs_stay_quiet_and_verbose_opens_up() {
    // A normal run must not narrate itself: the spinner is the user-facing
    // channel, and internal logs would fight it for the same stderr.
    assert_eq!(LogDetail::Normal.level(), tracing::Level::WARN);
    assert_eq!(LogDetail::Verbose.level(), tracing::Level::DEBUG);
}

#[test]
fn installing_twice_does_not_panic() {
    super::install(LogDetail::Normal);
    super::install(LogDetail::Verbose);
}
