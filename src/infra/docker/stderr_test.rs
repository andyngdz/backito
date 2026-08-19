use super::trailing_stderr;

#[test]
fn short_stderr_is_kept_whole_and_trimmed() {
    assert_eq!(trailing_stderr(b"  boom  \n"), "boom");
}

#[test]
fn long_stderr_keeps_the_tail() {
    let noisy = format!("{}the real error", "preamble ".repeat(500));

    let kept = trailing_stderr(noisy.as_bytes());

    assert!(kept.ends_with("the real error"));
    assert!(kept.len() < noisy.len());
}

#[test]
fn multibyte_stderr_is_cut_on_a_char_boundary() {
    // A naive byte slice here would panic or produce invalid UTF-8.
    let noisy = "đủ dấu ".repeat(500);

    let kept = trailing_stderr(noisy.as_bytes());

    // The result is a suffix of the trimmed input, cut where a character
    // starts -- never mid-codepoint.
    assert!(noisy.trim_end().ends_with(&kept));
    assert!(kept.starts_with(|glyph: char| glyph.is_alphabetic() || glyph == ' '));
}
