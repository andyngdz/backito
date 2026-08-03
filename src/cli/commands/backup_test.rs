use super::LocalCopy;

#[test]
fn the_two_dispositions_are_distinct() {
    // The dispatcher maps --keep onto these; nothing else may collapse them.
    assert_ne!(LocalCopy::Keep, LocalCopy::Discard);
}
