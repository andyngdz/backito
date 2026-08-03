use super::{TerminalReporter, human_bytes};
use crate::features::progress::{ProgressObserver, Step};

#[test]
fn byte_counts_render_through_the_progress_crate() {
    assert_eq!(human_bytes(0), "0 B");
    assert!(human_bytes(925_177_935).contains("MiB"));
}

#[test]
fn a_quiet_reporter_still_accepts_every_event() {
    // --quiet must change what is drawn, never what the services may call.
    let reporter = TerminalReporter::new(true);

    reporter.step_started(Step::Dump);
    reporter.transfer_started(Some(100));
    reporter.transfer_advanced(50);
    reporter.transfer_finished();
    reporter.step_finished(Step::Dump, "882 MiB");
    reporter.warn("scratch container left behind");
}

#[tokio::test]
async fn the_metered_reader_passes_bytes_through_unchanged() {
    use tokio::io::AsyncReadExt;

    let reporter = TerminalReporter::new(true);
    reporter.transfer_started(Some(8));
    let wrap = reporter.metered_reader();

    let file = tokio::fs::File::open("Cargo.toml").await.expect("open");
    let mut metered = wrap(file);
    let mut body = Vec::new();
    metered.read_to_end(&mut body).await.expect("read");

    let direct = std::fs::read("Cargo.toml").expect("read direct");
    assert_eq!(body, direct);
}
