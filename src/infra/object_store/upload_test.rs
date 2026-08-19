use super::{MAX_PARTS_IN_FLIGHT, PART_BYTES, write_all_parts};
use object_store::{ObjectStoreExt as _, WriteMultipart};
use std::sync::Arc;

/// Drives `write_all_parts` against an in-memory store and returns what landed.
///
/// The loop is the piece worth testing: it reads a fixed buffer, waits for
/// capacity, and hands over whatever it filled. A short read that is treated as
/// end-of-stream, or an off-by-one on the filled slice, both silently truncate
/// the archive, which is exactly the failure a backup tool must not have.
async fn upload_through_loop(body: &[u8]) -> Vec<u8> {
    let store = Arc::new(object_store::memory::InMemory::new());
    let location = object_store::path::Path::from("archive.dump");

    let upload = store.put_multipart(&location).await.expect("start upload");
    let mut writer = WriteMultipart::new_with_chunk_size(upload, PART_BYTES);

    let mut reader = std::io::Cursor::new(body.to_vec());
    write_all_parts("archive.dump", &mut writer, &mut reader)
        .await
        .expect("stream the body");
    writer.finish().await.expect("finish upload");

    store
        .get(&location)
        .await
        .expect("read back")
        .bytes()
        .await
        .expect("collect")
        .to_vec()
}

#[tokio::test]
async fn a_body_smaller_than_one_part_lands_whole() {
    let body = vec![7_u8; 1024];

    assert_eq!(upload_through_loop(&body).await, body);
}

#[tokio::test]
async fn a_body_spanning_several_parts_lands_whole_and_in_order() {
    // Past the in-flight ceiling, so `wait_for_capacity` actually blocks rather
    // than being a no-op the way it is for every small-file test.
    let parts = MAX_PARTS_IN_FLIGHT + 2;
    let body: Vec<u8> = (0..PART_BYTES * parts)
        .map(|index| (index % 251) as u8)
        .collect();

    let stored = upload_through_loop(&body).await;

    assert_eq!(stored.len(), body.len());
    assert_eq!(stored, body);
}

#[tokio::test]
async fn a_body_one_byte_past_a_part_boundary_keeps_the_last_byte() {
    // The classic truncation: a final chunk shorter than the read buffer must
    // still be written, and only the filled prefix of it.
    let body: Vec<u8> = (0..PART_BYTES + 1)
        .map(|index| (index % 251) as u8)
        .collect();

    let stored = upload_through_loop(&body).await;

    assert_eq!(stored.len(), PART_BYTES + 1);
    assert_eq!(stored, body);
}

#[tokio::test]
async fn an_empty_body_uploads_as_an_empty_object() {
    assert!(upload_through_loop(&[]).await.is_empty());
}
