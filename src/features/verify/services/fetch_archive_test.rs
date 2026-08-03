use super::describe;
use crate::features::verify::ChecksumOutcome;

#[test]
fn a_match_reads_as_confirmation() {
    assert_eq!(
        describe(&ChecksumOutcome::Matched),
        "matches stored checksum"
    );
}

#[test]
fn a_mismatch_shows_both_digests_so_the_gap_is_visible() {
    let described = describe(&ChecksumOutcome::Mismatched {
        expected: "aaa".to_owned(),
        actual: "bbb".to_owned(),
    });

    assert!(described.contains("aaa"));
    assert!(described.contains("bbb"));
    assert!(described.contains("MISMATCH"));
}

#[test]
fn an_absent_sidecar_says_so_rather_than_implying_a_pass() {
    let described = describe(&ChecksumOutcome::Absent);

    assert!(described.contains("no stored checksum"));
}
