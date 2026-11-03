use crate::ci::{test_support::*, *};
use axum::http::StatusCode;

#[tokio::test]
async fn file_commit_skip_pipeline_preserves_indexing_and_default_pipeline_admission() {
  let (_dir, state, _app, owner, project) = fixture().await;
  let path = format!("/projects/{project}/repository/file-commits");
  let app = crate::repository::routes().with_state(state.clone());
  assert!(state.ci.pipelines(project).await.unwrap().is_empty());
  // A supported source file makes subsequent saves eligible for code indexing.
  let (status, _) = request(
    &app,
    "POST",
    &path,
    Some(&owner),
    json!({
      "file_path":"src/lib.rs", "content":"pub fn hello() {}", "message":"Add source",
      "skip_pipeline":true
    }),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let head = state
    .git
    .resolve_revision(&format!("{project}.git"), "main", "")
    .await
    .unwrap();
  let yaml = "jobs:\n  check:\n    script: echo check\n";
  let body = json!({
    "file_path":".gity-ci.yml", "content":yaml, "message":"Save CI without running",
    "expected_head_sha":head, "update_existing":true, "skip_pipeline":true
  });
  let (status, saved) = request(&app, "POST", &path, Some(&owner), body).await;
  assert_eq!(status, StatusCode::OK, "{saved}");
  assert!(state.ci.pipelines(project).await.unwrap().is_empty());
  let saved_sha = text(&saved["data"], "commit_sha");
  let jobs = state.ci.jobs(project).await.unwrap();
  assert!(
    jobs.iter().any(|job| {
      job.kind == "index"
        && job.payload.as_ref().is_some_and(|payload| {
          let payload: Value = serde_json::from_str(payload).unwrap();
          payload["commit_sha"] == saved_sha
        })
    }),
    "saving with skip_pipeline must still enqueue code indexing"
  );
  // Omission preserves the existing behavior for normal repository edits.
  let (status, normal) = request(
    &app,
    "POST",
    &path,
    Some(&owner),
    json!({
      "file_path":".gity-ci.yml", "content":yaml, "message":"Normal CI commit",
      "expected_head_sha":saved_sha, "update_existing":true
    }),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{normal}");
  let pipelines = state.ci.pipelines(project).await.unwrap();
  assert_eq!(pipelines.len(), 1);
  assert_eq!(pipelines[0].source, "push");
  assert_eq!(
    pipelines[0].commit_sha.as_deref(),
    Some(text(&normal["data"], "commit_sha"))
  );
  assert_eq!(pipelines[0].config_content, yaml);
  assert_eq!(
    request(
      &app,
      "POST",
      &path,
      Some(&owner),
      json!({
        "file_path":"bad", "content":"invalid", "message":"Invalid flag", "skip_pipeline":"true"
      })
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );
}
