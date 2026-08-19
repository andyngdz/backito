//! Keeping the useful part of a failed command's stderr.

/// Bytes of stderr kept in an error message.
const STDERR_KEEP_BYTES: usize = 2000;

/// Keeps the tail of stderr, which is where the useful line is when a tool
/// prints a long preamble first. Cuts on a char boundary so the message stays
/// valid UTF-8.
pub fn trailing_stderr(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let trimmed = text.trim();
    if trimmed.len() <= STDERR_KEEP_BYTES {
        return trimmed.to_owned();
    }

    let cut_from = trimmed.len() - STDERR_KEEP_BYTES;
    let boundary = (cut_from..trimmed.len())
        .find(|index| trimmed.is_char_boundary(*index))
        .unwrap_or(trimmed.len());
    trimmed[boundary..].to_owned()
}

#[cfg(test)]
#[path = "stderr_test.rs"]
mod stderr_test;
