use super::{TerminalReporter, human_bytes};
use crate::features::progress::{ProgressObserver, Step};

#[test]
fn byte_counts_render_through_the_progress_crate() {
    assert_eq!(human_bytes(0), "0 B");
    assert!(human_bytes(925_177_935).contains("MiB"));
}

#[test]
fn the_reporter_accepts_every_event_a_service_can_send() {
    // Tests capture stderr, so this asserts the reporter survives the full
    // event sequence rather than what it drew. What reaches a terminal is
    // decided by indicatif, which renders nothing without a TTY.
    let reporter = TerminalReporter::new();

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

    let reporter = TerminalReporter::new();
    reporter.transfer_started(Some(8));
    let wrap = reporter.metered_reader();

    let file = tokio::fs::File::open("Cargo.toml").await.expect("open");
    let mut metered = wrap(file);
    let mut body = Vec::new();
    metered.read_to_end(&mut body).await.expect("read");

    let direct = std::fs::read("Cargo.toml").expect("read direct");
    assert_eq!(body, direct);
}

/// A line sink the test can read back after the reporter has written to it.
#[derive(Clone, Default)]
struct SharedBuffer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for SharedBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("buffer lock").extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn lines_reach_the_sink_when_the_bar_is_hidden() {
    // A container's stderr is not a terminal, so the bar is hidden. Lines
    // written through it vanished there, and so did every wal-g warning.
    let sink = SharedBuffer::default();
    let reporter = TerminalReporter::with_lines(Box::new(sink.clone()));

    reporter.info("base backup every 24h");
    reporter.step_started(Step::Dump);
    reporter.step_finished(Step::Dump, "882 MiB");
    reporter.warn("base backup failed");

    let written = String::from_utf8(sink.0.lock().expect("buffer lock").clone()).expect("utf8");
    assert!(written.contains("- base backup every 24h\n"), "{written}");
    assert!(written.contains("882 MiB\n"), "{written}");
    assert!(written.contains("! base backup failed\n"), "{written}");
}
