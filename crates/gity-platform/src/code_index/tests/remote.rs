//! The object-storage copy of code-index archives.
//!
//! These tests use the local `Storage` driver, which is the same code path as
//! S3 behind the `Storage` abstraction: a missing local index is restored from
//! the shared copy, and an unreachable store only degrades.

use super::remote;
use crate::AppState;
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
use gity_persistence::model_value;
use serde_json::json;

const SOURCE: &str = "pub fn run() {}\n";

async fn fixture() -> (tempfile::TempDir, AppState) {
  crate::features::test_support::fixture().await
}

fn archive(commit: &str) -> gity_code_intel::Archive {
  let file = gity_code_intel::Language::Rust
    .parse("src/app.rs", SOURCE)
    .unwrap();
  gity_code_intel::Archive::new(commit, vec![file])
}

async fn commit(s: &AppState, project: i64, path: &str, content: &str) -> String {
  s.git
    .create_file_commit(
      &format!("{project}.git"),
      gity_git::CreateFileCommitInput {
        expected_head_sha: None,
        update_existing: false,
        branch_name: "main".into(),
        file_path: path.into(),
        content: content.into(),
        message: format!("add {path}"),
        author_name: "Owner".into(),
        author_email: "owner@example.test".into(),
      },
    )
    .await
    .unwrap()
}

fn authorization(token: &str) -> HeaderMap {
  let mut headers = HeaderMap::new();
  headers.insert("authorization", format!("Bearer {token}").parse().unwrap());
  headers
}

#[test]
fn archive_keys_name_the_project_and_commit() {
  assert_eq!(
    remote::archive_key(7, "abc123").as_deref(),
    Some("code-index/7/abc123.msgpack.zst")
  );
  assert!(remote::archive_key(0, "abc123").is_none());
  assert!(remote::archive_key(-1, "abc123").is_none());
  assert!(remote::archive_key(7, "").is_none());
  assert!(remote::archive_key(7, "refs/heads/main").is_none());
  assert!(remote::archive_key(7, &"a".repeat(65)).is_none());
}

#[tokio::test]
async fn a_missing_local_index_is_restored_from_the_shared_archive() {
  let (_dir, s) = fixture().await;
  let source = archive("aaa111");
  remote::publish_archive(&s, 20, &source).await;
  assert!(s.code_index.status(20).unwrap().is_none(), "start empty");

  assert!(remote::hydrate(&s, 20, "aaa111").await);

  let status = s.code_index.status(20).unwrap().expect("index restored");
  assert_eq!(status.commit, "aaa111");
  let rows = s.code_index.symbols(20, "src/app.rs").unwrap();
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].name, "run");
}

#[tokio::test]
async fn a_stale_local_commit_is_replaced_by_the_shared_archive() {
  let (_dir, s) = fixture().await;
  let store = s.code_index.clone();
  store.refresh(20, &archive("eee555")).unwrap();
  remote::publish_archive(&s, 20, &archive("fff666")).await;

  assert!(remote::hydrate(&s, 20, "fff666").await);
  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, "fff666");
}

#[tokio::test]
async fn a_matching_local_index_never_reads_object_storage() {
  let (_dir, s) = fixture().await;
  let store = s.code_index.clone();
  store.refresh(20, &archive("bbb222")).unwrap();
  let reads = remote::remote_reads();

  assert!(remote::hydrate(&s, 20, "bbb222").await);
  assert_eq!(
    remote::remote_reads(),
    reads,
    "a served commit must not trigger an object read"
  );
}

#[tokio::test]
async fn an_unreachable_shared_store_only_degrades() {
  let (dir, s) = fixture().await;
  // A regular file where the key prefix belongs makes every store call fail.
  std::fs::write(dir.path().join("storage").join(remote::PREFIX), b"blocked").unwrap();
  let reads = remote::remote_reads();

  assert!(!remote::hydrate(&s, 20, "ccc333").await);
  assert!(remote::remote_reads() > reads, "the read was attempted");

  // The local pipeline keeps working while the shared copy is down.
  let store = s.code_index.clone();
  store.refresh(20, &archive("ddd444")).unwrap();
  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, "ddd444");
  remote::publish_archive(&s, 20, &archive("ddd444")).await;
  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, "ddd444");
}

#[tokio::test]
async fn ingest_mirrors_the_uploaded_artifact_bytes() {
  let (_dir, s) = fixture().await;
  let bytes = gity_code_intel::encode(&archive("abc123")).unwrap();
  let artifact_key = "ci/20/1/code-index";
  s.storage
    .put(artifact_key, bytes::Bytes::from(bytes.clone()))
    .await
    .unwrap();
  let job = s
    .ci
    .create_system_job(gity_ci::NewJob {
      project_id: 20,
      kind: "index".into(),
      payload: json!({
        "commit_sha":"abc123",
        "artifact_name":crate::code_index::ARTIFACT_NAME
      })
      .to_string(),
      max_attempts: 2,
      run_after: chrono::Utc::now().naive_utc(),
    })
    .await
    .unwrap();
  s.ci
    .add_artifact(gity_ci::NewArtifact {
      id: gity_core::new_id(),
      project_id: 20,
      job_id: job.id,
      name: crate::code_index::ARTIFACT_NAME.into(),
      file_name: crate::code_index::ARTIFACT_NAME.into(),
      file_path: crate::code_index::ARTIFACT_NAME.into(),
      content_type: "application/octet-stream".into(),
      byte_size: bytes.len() as i64,
      storage_key: artifact_key.into(),
    })
    .await
    .unwrap();

  crate::code_index::ingest(&s, 20, &model_value(job))
    .await
    .unwrap();

  let key = remote::archive_key(20, "abc123").expect("archive key");
  let stored = s.storage.get(&key).await.expect("mirrored archive");
  assert_eq!(stored.as_ref(), bytes.as_slice());
  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, "abc123");
}

#[tokio::test]
async fn the_default_branch_head_is_restored_for_a_read() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  let sha = commit(&s, 20, "src/lib.rs", SOURCE).await;
  // Another replica indexed this commit and published the archive.
  let source = gity_code_intel::Archive::new(
    sha.clone(),
    vec![
      gity_code_intel::Language::Rust
        .parse("src/lib.rs", SOURCE)
        .unwrap(),
    ],
  );
  remote::publish_archive(&s, 20, &source).await;
  assert!(s.code_index.status(20).unwrap().is_none());

  let project = gity_persistence::model_value(s.ci.project(20).await.unwrap());
  assert!(remote::hydrate_head(&s, &project).await);

  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, sha);
  assert_eq!(s.code_index.symbols(20, "src/lib.rs").unwrap().len(), 1);
}

#[tokio::test]
async fn a_manual_rebuild_mirrors_the_fresh_archive() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  let sha = commit(&s, 20, "src/lib.rs", SOURCE).await;

  // The handler is called directly: assembling the whole router is unrelated to
  // this behaviour and currently broken by another change in the tree.
  let reply = super::refresh::refresh(State(s.clone()), authorization("alice"), Path(20)).await;
  assert!(reply.is_ok(), "rebuild failed: {:?}", reply.err());

  let key = remote::archive_key(20, &sha).expect("archive key");
  assert!(s.storage.exists(&key).await.unwrap(), "rebuild must mirror");
  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, sha);
}
