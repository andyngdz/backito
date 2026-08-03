//! What a finished command hands back to the entrypoint.

use super::ExitStatus;

/// A command's result: what to print, and how the process should end.
///
/// Commands return this instead of printing, so the entrypoint owns stdout and
/// a command stays testable without capturing output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandReport {
    /// Lines for stdout, in order.
    pub lines: Vec<String>,
    /// Status the shell should see.
    pub status: ExitStatus,
}

impl CommandReport {
    /// A successful command whose result is one line, such as an object key.
    pub fn line(value: impl Into<String>) -> Self {
        Self {
            lines: vec![value.into()],
            status: ExitStatus::Success,
        }
    }

    /// A command that produced a multi-line report and its own status.
    pub fn lines(lines: Vec<String>, status: ExitStatus) -> Self {
        Self { lines, status }
    }
}

#[cfg(test)]
#[path = "dto_test.rs"]
mod dto_test;
