use crate::{
  ArtifactCoordinate, ArtifactRepository, ArtifactService, PackageFormat, RecordArtifactUpload,
};
use bytes::Bytes;
use gity_core::{Db, Storage};

#[tokio::test]
async fn deleting_file_commits_outbox_then_cleans_object_storage() {
  let database = Db::connect("sqlite::memory:").await.unwrap();
  database.migrate().await.unwrap();
  database
    .insert(
      "organizations",
      serde_json::from_value(serde_json::json!({
          "id":10,"name":"Team","path_key":"team","full_path":"team","visibility":"private"
      }))
      .unwrap(),
    )
    .await
    .unwrap();
  database.insert("projects", serde_json::from_value(serde_json::json!({
        "id":20,"organization_id":10,"name":"Project","path_key":"project",
        "full_path":"team/project","visibility":"private","default_branch":"main","status":"active"
    })).unwrap()).await.unwrap();
  let directory = tempfile::tempdir().unwrap();
  let storage = Storage::local(directory.path()).unwrap();
  let key = "packages/20/file-1";
  storage
    .put(key, Bytes::from_static(b"payload"))
    .await
    .unwrap();
  let repository = ArtifactRepository::new(database.connection.clone());
  let coordinate = ArtifactCoordinate::new(
    20,
    PackageFormat::Generic,
    "widget",
    "1.0",
    "nested/file.bin",
  )
  .unwrap();
  let (file, created) = repository
    .record_upload(RecordArtifactUpload {
      coordinate: &coordinate,
      file_name: "file.bin",
      content_type: "application/octet-stream",
      byte_size: 7,
      sha256: "239f59ed55e737c77147cf55ad0c1b030b6d7ee748a7426952f9b852d5a935e5",
      storage_key: key,
      created_by_user_id: Some(10),
      source: "manual",
      pipeline_id: None,
      job_id: None,
      registry_metadata: None,
    })
    .await
    .unwrap();
  assert!(created);
  repository.delete_file(20, file.id).await.unwrap();
  assert!(storage.exists(key).await.unwrap());

  let service = ArtifactService {
    repository,
    storage,
  };
  assert_eq!(service.cleanup_pending(100).await.unwrap(), 1);
  assert!(!service.storage.exists(key).await.unwrap());
}
