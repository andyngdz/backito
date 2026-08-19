//! Noticing that the process has been asked to stop.
//!
//! Unix only, because the signal that matters is SIGTERM: that is what `docker
//! stop` and a systemd unit send, and both are how this tool is actually
//! stopped. Without handling it the process is killed where it stands, which
//! leaves the scratch container up and the pass's multi-GB dump on disk.

use tokio::signal::unix::{SignalKind, signal};

/// What both scheduling loops report when they are asked to stop.
pub const STOPPING: &str = "stopping";

/// Why a wait ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Woke {
    /// The wait ran its course. Carry on.
    Elapsed,
    /// A stop was requested. Unwind and let the guards run.
    Stopping,
}

/// Resolves the first time the process is asked to stop.
///
/// A stream that cannot be registered is treated as a signal that will never
/// arrive rather than a failure: refusing to run a scheduler because it could
/// not arrange to shut down tidily trades a real backup for a tidy exit.
pub async fn requested() {
    let mut terminate = listen(SignalKind::terminate());
    let mut interrupt = listen(SignalKind::interrupt());

    match (terminate.as_mut(), interrupt.as_mut()) {
        (Some(term), Some(int)) => {
            tokio::select! {
                _ = term.recv() => {}
                _ = int.recv() => {}
            }
        }
        (Some(only), None) | (None, Some(only)) => {
            only.recv().await;
        }
        (None, None) => std::future::pending().await,
    }
}

/// Waits `duration`, returning early when a stop is requested.
pub async fn sleep_unless_stopped(duration: std::time::Duration) -> Woke {
    tokio::select! {
        () = tokio::time::sleep(duration) => Woke::Elapsed,
        () = requested() => Woke::Stopping,
    }
}

/// Runs `work` unless a stop arrives first.
///
/// Dropping the unfinished future is what stops the work: the commands it is
/// waiting on are spawned with `kill_on_drop`, so the child goes with it rather
/// than outliving the process that started it.
pub async fn unless_stopped<F: Future>(work: F) -> Option<F::Output> {
    tokio::select! {
        done = work => Some(done),
        () = requested() => None,
    }
}

/// Registers one signal stream, or `None` when the platform refuses.
fn listen(kind: SignalKind) -> Option<tokio::signal::unix::Signal> {
    signal(kind)
        .map_err(|source| {
            tracing::warn!(%source, "could not listen for a stop signal, shutdown will be abrupt");
        })
        .ok()
}

#[cfg(test)]
#[path = "shutdown_test.rs"]
mod shutdown_test;
