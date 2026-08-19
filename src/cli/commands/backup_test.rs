use super::LocalCopy;

#[test]
fn the_keep_flag_maps_onto_the_disposition_it_names() {
    // This is what the dispatcher calls. Getting it backwards deletes the local
    // archive of someone who asked to keep it.
    assert_eq!(LocalCopy::from(true), LocalCopy::Keep);
    assert_eq!(LocalCopy::from(false), LocalCopy::Discard);
}
