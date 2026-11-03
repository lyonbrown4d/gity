use crate::AppState;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use gity_persistence::model_value;
use serde_json::{Value, json};
use tower::ServiceExt;

const SOURCE: &str = "pub fn run() {}\n";

async fn fixture() -> (tempfile::TempDir, AppState) {
  crate::features::test_support::fixture().await
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

async fn project(s: &AppState, id: i64) -> Value {
  model_value(s.ci.project(id).await.unwrap())
}

async fn post(app: &axum::Router, path: &str, token: &str) -> (StatusCode, Value) {
  let request = Request::builder()
    .method("POST")
    .uri(path)
    .header("content-type", "application/json")
    .header("authorization", format!("Bearer {token}"))
    .body(Body::from("{}"))
    .unwrap();
  let response = app.clone().oneshot(request).await.unwrap();
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 20 * 1024 * 1024)
    .await
    .unwrap();
  (
    status,
    serde_json::from_slice(&bytes).unwrap_or(Value::Null),
  )
}

async fn index_jobs(s: &AppState, project: i64) -> Vec<Value> {
  s.ci
    .jobs(project)
    .await
    .unwrap()
    .into_iter()
    .map(model_value)
    .filter(|row| row["kind"] == "index")
    .collect()
}

#[tokio::test]
async fn default_branch_push_with_rust_sources_enqueues_an_index_job() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  let sha = commit(&s, 20, "src/lib.rs", SOURCE).await;
  crate::events::repository_changed(&s, &project(&s, 20).await, "refs/heads/main", &sha).await;
  let jobs = index_jobs(&s, 20).await;
  assert_eq!(jobs.len(), 1, "{jobs:?}");
  assert_eq!(jobs[0]["status"], "pending");
  let payload: Value = serde_json::from_str(jobs[0]["payload"].as_str().unwrap()).unwrap();
  assert_eq!(payload["commit_sha"], sha);
  assert_eq!(payload["ref_name"], "main");
  assert_eq!(payload["tags"], json!(["gity-indexer"]));
  assert_eq!(payload["artifact_name"], "code-index.msgpack.zst");
}

#[tokio::test]
async fn pushes_without_supported_sources_enqueue_nothing() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  let sha = commit(&s, 20, "README.md", "no code here\n").await;
  crate::events::repository_changed(&s, &project(&s, 20).await, "refs/heads/main", &sha).await;
  assert!(index_jobs(&s, 20).await.is_empty());
}

#[tokio::test]
async fn non_default_branch_pushes_enqueue_nothing() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  commit(&s, 20, "src/lib.rs", SOURCE).await;
  s.git
    .create_file_commit(
      "20.git",
      gity_git::CreateFileCommitInput {
        expected_head_sha: None,
        update_existing: false,
        branch_name: "feature".into(),
        file_path: "src/extra.rs".into(),
        content: SOURCE.into(),
        message: "feature".into(),
        author_name: "Owner".into(),
        author_email: "owner@example.test".into(),
      },
    )
    .await
    .unwrap();
  let sha = s
    .git
    .resolve_revision("20.git", "feature", "")
    .await
    .unwrap();
  crate::events::repository_changed(&s, &project(&s, 20).await, "refs/heads/feature", &sha).await;
  assert!(index_jobs(&s, 20).await.is_empty());
}

#[tokio::test]
async fn ingest_writes_the_uploaded_archive_into_the_store() {
  let (_dir, s) = fixture().await;
  let file = gity_code_intel::Language::Rust
    .parse("src/app.rs", SOURCE)
    .unwrap();
  let bytes =
    gity_code_intel::encode(&gity_code_intel::Archive::new("abc123", vec![file])).unwrap();
  let key = "ci/20/1/code-index";
  s.storage
    .put(key, bytes::Bytes::from(bytes.clone()))
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
      storage_key: key.into(),
    })
    .await
    .unwrap();
  crate::code_index::ingest(&s, 20, &model_value(job))
    .await
    .unwrap();
  let status = s.code_index.status(20).unwrap().unwrap();
  assert_eq!(status.commit, "abc123");
  assert_eq!(status.files, 1);
  assert!(status.definitions >= 1);
}

#[tokio::test]
async fn manual_refresh_scans_the_default_branch_head() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  let sha = commit(&s, 20, "src/lib.rs", SOURCE).await;
  let app = crate::router(s.clone());
  let (status, body) = post(
    &app,
    "/api/v1/projects/20/search/code-index/refresh",
    "alice",
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{body}");
  let data = &body["data"];
  assert_eq!(data["commit"], sha);
  assert_eq!(data["files"], 1);
  assert!(data["definitions"].as_u64().unwrap_or_default() >= 1);
  assert!(!data["indexed_at"].as_str().unwrap_or_default().is_empty());
  assert_eq!(s.code_index.status(20).unwrap().unwrap().commit, sha);
}

#[tokio::test]
async fn manual_refresh_requires_repository_admin() {
  let (_dir, s) = fixture().await;
  s.git.init_bare("20.git", "main").await.unwrap();
  commit(&s, 20, "src/lib.rs", SOURCE).await;
  let app = crate::router(s.clone());
  let (status, _) = post(
    &app,
    "/api/v1/projects/20/search/code-index/refresh",
    "outsider",
  )
  .await;
  assert_eq!(status, StatusCode::FORBIDDEN);
  assert!(s.code_index.status(20).unwrap().is_none());
}

#[test]
fn snapshot_extraction_rejects_unsafe_paths() {
  use std::io::Write;
  let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
  zip
    .start_file("../escape", zip::write::SimpleFileOptions::default())
    .unwrap();
  zip.write_all(b"bad").unwrap();
  let dir = tempfile::tempdir().unwrap();
  let bytes = zip.finish().unwrap().into_inner();
  assert!(crate::code_index::snapshot::extract(std::io::Cursor::new(bytes), dir.path()).is_err());
}
