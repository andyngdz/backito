//! Internal log output, separate from the progress a user watches.

/// How much internal detail goes to stderr.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogDetail {
    /// Warnings and above. The default: internal logs are for diagnosing a
    /// failure, not for narrating a normal run.
    Normal,
    /// Everything, for diagnosing a failure.
    Verbose,
}

impl LogDetail {
    /// The tracing level this detail maps to.
    pub fn level(self) -> tracing::Level {
        match self {
            Self::Normal => tracing::Level::WARN,
            Self::Verbose => tracing::Level::DEBUG,
        }
    }
}

/// Installs the log subscriber on stderr.
///
/// Ignores a second call: the subscriber is process-global, and failing here
/// would take down a command over its logging.
pub fn install(detail: LogDetail) {
    let _ = tracing_subscriber::fmt()
        .with_max_level(detail.level())
        .with_writer(std::io::stderr)
        .try_init();
}

#[cfg(test)]
#[path = "logging_test.rs"]
mod logging_test;
