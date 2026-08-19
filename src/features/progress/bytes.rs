//! Rendering a byte count for a human to read.

use indicatif::HumanBytes;

/// Formats `bytes` in binary units, e.g. `882.34 MiB`.
///
/// Every size a user sees comes through here. Two formatters were in use before,
/// one per layer, which meant the same number could be printed two ways in a
/// single run depending on which step reported it.
pub fn human_bytes(bytes: u64) -> String {
    HumanBytes(bytes).to_string()
}

#[cfg(test)]
#[path = "bytes_test.rs"]
mod bytes_test;
