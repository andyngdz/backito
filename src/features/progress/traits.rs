//! The contract a command's services report progress through.
//!
//! Services own the work; the interface layer owns how it looks. Everything
//! here is display-agnostic, so a spinner, a quiet run, and a log stream are the
//! same code path with a different observer.

use std::sync::Arc;
use tokio::io::AsyncRead;

use super::Step;

/// Wraps an upload source so the interface layer can meter it.
///
/// Transfer progress has to come from the reader the S3 client pulls from, and
/// only the interface layer knows how that should be displayed. Boxing keeps
/// this usable behind `dyn`.
pub type MeteredReader =
    Arc<dyn Fn(tokio::fs::File) -> Box<dyn AsyncRead + Unpin + Send> + Send + Sync>;

/// Receives progress events as work happens.
pub trait ProgressObserver: Send + Sync {
    /// A step began.
    fn step_started(&self, step: Step);

    /// A step finished. `detail` is a short result worth keeping on screen,
    /// such as a size or a count; empty when there is nothing to add.
    fn step_finished(&self, step: Step, detail: &str);

    /// A byte transfer began within the current step. `total` is `None` when
    /// the size is not known ahead of time.
    fn transfer_started(&self, total: Option<u64>);

    /// More bytes moved.
    fn transfer_advanced(&self, bytes: u64);

    /// The byte transfer ended.
    fn transfer_finished(&self);

    /// Something worth telling the user that is not a failure, such as a
    /// scratch container left behind by an earlier interrupted run.
    fn warn(&self, message: &str);

    /// Routine progress from a long-running command, such as the daemon saying
    /// what a pass did. Defaulted to silence: a one-shot command already reports
    /// through the step callbacks and would only repeat itself.
    fn info(&self, _message: &str) {}

    /// Returns a wrapper that meters an upload source.
    fn metered_reader(&self) -> MeteredReader;
}

/// An observer that reports nothing, used by tests that exercise a service
/// without a terminal.
#[cfg(test)]
pub struct SilentObserver;

#[cfg(test)]
impl ProgressObserver for SilentObserver {
    fn step_started(&self, _step: Step) {}
    fn step_finished(&self, _step: Step, _detail: &str) {}
    fn transfer_started(&self, _total: Option<u64>) {}
    fn transfer_advanced(&self, _bytes: u64) {}
    fn transfer_finished(&self) {}
    fn warn(&self, _message: &str) {}

    fn metered_reader(&self) -> MeteredReader {
        Arc::new(|file| Box::new(file))
    }
}
