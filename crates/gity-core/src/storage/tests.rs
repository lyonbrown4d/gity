use super::*;

#[tokio::test]
async fn local_storage_streams_ranges_and_rejects_path_escape() {
  let dir = tempfile::tempdir().unwrap();
  let storage = Storage::local(dir.path()).unwrap();
  verify_contract(&storage).await;
  assert!(storage.put("../escape", Bytes::new()).await.is_err());
  #[cfg(unix)]
  {
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), dir.path().join("escape")).unwrap();
    assert!(storage.put("escape/file", Bytes::new()).await.is_err());
  }
}

#[tokio::test]
async fn seaweedfs_s3_contract() {
  let Ok(endpoint) = std::env::var("GITY_TEST_S3_ENDPOINT") else {
    return;
  };
  assert!(
    endpoint.starts_with("http://localhost:") || endpoint.starts_with("http://127.0.0.1:"),
    "S3 integration tests require a disposable localhost endpoint"
  );
  let mut settings = crate::Settings::load();
  settings.storage.driver = "s3".into();
  settings.storage.s3_endpoint = endpoint;
  settings.storage.s3_bucket = std::env::var("GITY_TEST_S3_BUCKET").unwrap_or("gity".into());
  settings.storage.s3_access_key =
    std::env::var("GITY_TEST_S3_ACCESS_KEY").unwrap_or("gitylocal".into());
  settings.storage.s3_secret_key =
    std::env::var("GITY_TEST_S3_SECRET_KEY").unwrap_or("gity-storage-local-only".into());
  settings.storage.s3_use_path_style = true;
  let storage = Storage::from_settings(&settings).unwrap();
  verify_contract(&storage).await;
  settings.storage.s3_secret_key = "incorrect-secret".into();
  let denied = Storage::from_settings(&settings).unwrap();
  assert!(
    denied
      .put(
        &format!("tests/denied-{}", crate::new_id()),
        Bytes::from_static(b"forbidden")
      )
      .await
      .is_err()
  );
}

async fn verify_contract(storage: &Storage) {
  let key = format!("tests/{}", crate::new_id());
  let chunks = vec![
    Ok(Bytes::from(vec![b'a'; 8 * 1024 * 1024])),
    Ok(Bytes::from(vec![b'b'; 1024 * 1024])),
  ];
  assert_eq!(
    storage
      .put_stream(&key, futures_util::stream::iter(chunks), 10 * 1024 * 1024)
      .await
      .unwrap(),
    9 * 1024 * 1024
  );
  assert_eq!(storage.head(&key).await.unwrap().size, 9 * 1024 * 1024);
  assert_eq!(
    storage
      .get_range(&key, 8 * 1024 * 1024 - 2..8 * 1024 * 1024 + 2)
      .await
      .unwrap(),
    Bytes::from_static(b"aabb")
  );
  let mut stream = storage.stream(&key).await.unwrap();
  let mut total = 0;
  while let Some(chunk) = stream.next().await {
    total += chunk.unwrap().len();
  }
  assert_eq!(total, 9 * 1024 * 1024);
  storage.delete(&key).await.unwrap();
  assert!(!storage.exists(&key).await.unwrap());
}
