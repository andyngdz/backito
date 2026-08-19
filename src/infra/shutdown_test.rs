use super::{Woke, sleep_unless_stopped, unless_stopped};
use std::time::Duration;

// The stop path itself is not covered here. A signal is delivered to the whole
// process, and `cargo test` runs every test in one, so raising SIGTERM to prove
// the wait ends early also ends every other test's wait. What is covered is the
// composition around it: that a wait left alone still elapses, and that work
// finishing first is what the caller gets back.

#[tokio::test]
async fn a_wait_that_runs_its_course_reports_elapsed() {
    let woke = sleep_unless_stopped(Duration::from_millis(1)).await;

    assert_eq!(woke, Woke::Elapsed);
}

#[tokio::test]
async fn work_that_finishes_first_hands_back_its_value() {
    let done = unless_stopped(async { 7_u8 }).await;

    assert_eq!(done, Some(7));
}

#[tokio::test]
async fn slow_work_still_returns_its_value_when_nothing_stops_it() {
    let done = unless_stopped(async {
        tokio::time::sleep(Duration::from_millis(5)).await;
        "finished"
    })
    .await;

    assert_eq!(done, Some("finished"));
}
