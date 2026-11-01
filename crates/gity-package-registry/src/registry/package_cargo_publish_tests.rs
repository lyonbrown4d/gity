use super::*;
use futures_util::TryStreamExt;

fn publish_frame(metadata: &[u8], archive: &[u8]) -> Bytes {
  let mut body = Vec::new();
  body.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
  body.extend_from_slice(metadata);
  body.extend_from_slice(&(archive.len() as u32).to_le_bytes());
  body.extend_from_slice(archive);
  Bytes::from(body)
}

#[tokio::test]
async fn cargo_publish_body_reads_length_prefixed_metadata_and_streams_archive() {
  let metadata = br#"{"name":"sample","vers":"1.2.3","deps":[]}"#;
  let archive = b"compressed crate bytes";
  let (metadata, archive_stream) =
    CargoPublishBody::parse(Body::from(publish_frame(metadata, archive)))
      .await
      .unwrap();

  assert_eq!(metadata["name"], "sample");
  let chunks = archive_stream.try_collect::<Vec<_>>().await.unwrap();
  assert_eq!(chunks.concat(), archive);
}

#[tokio::test]
async fn cargo_publish_body_rejects_truncated_and_trailing_archive_bytes() {
  let metadata = br#"{"name":"sample","vers":"1.2.3"}"#;
  let truncated = publish_frame(metadata, b"short");
  let truncated = truncated.slice(..truncated.len() - 1);
  let (_, stream) = CargoPublishBody::parse(Body::from(truncated))
    .await
    .unwrap();
  assert!(stream.try_collect::<Vec<_>>().await.is_err());

  let mut trailing = publish_frame(metadata, b"short").to_vec();
  trailing.push(b'!');
  let (_, stream) = CargoPublishBody::parse(Body::from(trailing)).await.unwrap();
  assert!(stream.try_collect::<Vec<_>>().await.is_err());
}

#[tokio::test]
async fn cargo_publish_body_rejects_oversized_metadata_before_reading_archive() {
  let mut body = (MAX_METADATA_BYTES as u32 + 1).to_le_bytes().to_vec();
  body.extend_from_slice(b"unused");
  assert!(CargoPublishBody::parse(Body::from(body)).await.is_err());
}
