//! Turns progress events into terminal output.
//!
//! Everything here writes to stderr. stdout carries only the command's result,
//! so `backito backup > key.txt` yields a usable key and a piped run shows its
//! progress in the terminal rather than in the file.

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::io::{self, Write};
use std::sync::Mutex;
use std::time::Duration;

use crate::features::progress::{MeteredReader, ProgressObserver, Step};

/// How often the spinner redraws.
const TICK_INTERVAL: Duration = Duration::from_millis(80);

/// Spinner template: a spinner, then what is happening.
const SPINNER_TEMPLATE: &str = "{spinner:.cyan} {msg}";

/// Transfer template: bar, transferred/total, and rate.
const TRANSFER_TEMPLATE: &str =
    "{spinner:.cyan} {msg} [{bar:24.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})";

/// Bar glyphs, from filled to empty.
const BAR_GLYPHS: &str = "=> ";

/// Mark printed beside a finished step.
const DONE_MARK: &str = "✓";

/// Mark printed beside a warning.
const WARN_MARK: &str = "!";

/// Marker on a routine line from a long-running command.
const INFO_MARK: &str = "-";

/// Draws steps as spinner lines, and transfers as a byte bar.
///
/// There is no quiet mode. Progress already suppresses itself when stderr is
/// not a terminal, which is the case a scheduled run cares about, so a flag
/// would only be able to hide the warnings and errors that a cron mail exists
/// to deliver.
///
/// Lines (finished steps, warnings, info) do not go through the bar, because
/// indicatif drops `println` on a hidden bar, and a bar is hidden whenever
/// stderr is not a terminal. They go to `lines` instead, with the bar
/// suspended so a live spinner does not draw over them.
pub struct TerminalReporter {
    bar: Mutex<ProgressBar>,
    lines: Mutex<Box<dyn Write + Send>>,
}

impl TerminalReporter {
    /// Builds a reporter drawing to stderr.
    pub fn new() -> Self {
        Self::with_lines(Box::new(io::stderr()))
    }

    /// Builds a reporter whose lines go to `lines` rather than stderr.
    pub(crate) fn with_lines(lines: Box<dyn Write + Send>) -> Self {
        Self {
            bar: Mutex::new(ProgressBar::hidden()),
            lines: Mutex::new(lines),
        }
    }

    /// Writes one line, suspending the bar around it.
    ///
    /// A poisoned lock means another thread panicked mid-draw; the line still
    /// has to reach the user, so take the inner value either way.
    fn print_line(&self, line: &str) {
        let current = self
            .bar
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut sink = self
            .lines
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let written = current.suspend(|| writeln!(sink, "{line}"));
        if written.is_err() {
            // The sink is gone; a terminal-attached bar can still draw it.
            current.println(line);
        }
    }

    /// Replaces the live bar, finishing whatever was there.
    fn swap(&self, next: ProgressBar) {
        if let Ok(mut current) = self.bar.lock() {
            current.finish_and_clear();
            *current = next;
        }
    }

    /// The draw target. `indicatif` renders nothing here when stderr is not a
    /// terminal, so a scheduled run needs no flag to stay clean.
    fn target(&self) -> ProgressDrawTarget {
        ProgressDrawTarget::stderr()
    }
}

impl ProgressObserver for TerminalReporter {
    fn step_started(&self, step: Step) {
        let spinner = ProgressBar::with_draw_target(None, self.target());
        spinner.set_style(
            ProgressStyle::with_template(SPINNER_TEMPLATE)
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        spinner.set_message(step.label().to_owned());
        spinner.enable_steady_tick(TICK_INTERVAL);
        self.swap(spinner);
    }

    fn step_finished(&self, step: Step, detail: &str) {
        let line = if detail.is_empty() {
            format!("{DONE_MARK} {}", step.label())
        } else {
            format!("{DONE_MARK} {} - {detail}", step.label())
        };

        self.print_line(&line);
        if let Ok(current) = self.bar.lock() {
            current.finish_and_clear();
        }
    }

    fn transfer_started(&self, total: Option<u64>) {
        let message = self
            .bar
            .lock()
            .map(|current| current.message())
            .unwrap_or_default();

        let bar = ProgressBar::with_draw_target(total, self.target());
        bar.set_style(
            ProgressStyle::with_template(TRANSFER_TEMPLATE)
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars(BAR_GLYPHS),
        );
        bar.set_message(message);
        bar.enable_steady_tick(TICK_INTERVAL);
        self.swap(bar);
    }

    fn transfer_advanced(&self, bytes: u64) {
        if let Ok(current) = self.bar.lock() {
            current.inc(bytes);
        }
    }

    fn transfer_finished(&self) {
        if let Ok(current) = self.bar.lock() {
            current.finish_and_clear();
        }
    }

    fn warn(&self, message: &str) {
        self.print_line(&format!("{WARN_MARK} {message}"));
    }

    fn info(&self, message: &str) {
        self.print_line(&format!("{INFO_MARK} {message}"));
    }

    fn metered_reader(&self) -> MeteredReader {
        let bar = self
            .bar
            .lock()
            .map(|current| current.clone())
            .unwrap_or_else(|_| ProgressBar::hidden());

        std::sync::Arc::new(move |file| Box::new(bar.clone().wrap_async_read(file)))
    }
}

/// Renders a byte count for a person.
pub fn human_bytes(bytes: u64) -> String {
    crate::features::progress::human_bytes(bytes)
}

#[cfg(test)]
#[path = "reporter_test.rs"]
mod reporter_test;
