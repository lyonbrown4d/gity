use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
pub(super) fn file(branch: &str, path: &str, content: &str) -> gity_git::CreateFileCommitInput {
  gity_git::CreateFileCommitInput {
    expected_head_sha: None,
    update_existing: false,
    branch_name: branch.into(),
    file_path: path.into(),
    content: content.into(),
    message: format!("Add {path}"),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
  }
}
#[tokio::test]
async fn merge_requires_current_commit_pipeline_and_non_author_approval() {
  let (_dir, s) = fixture().await;
  let repo = "20.git";
  s.git.init_bare(repo, "main").await.unwrap();
  s.git
    .create_file_commit(repo, file("main", "README.md", "Base"))
    .await
    .unwrap();
  s.git.create_branch(repo, "feature", "main").await.unwrap();
  s.git
    .create_file_commit(
      repo,
      file("feature", ".gity-ci.yml", "test:\n  script: echo ok\n"),
    )
    .await
    .unwrap();
  let created = request(
    &s,
    "POST",
    "/projects/20/merge-requests",
    "alice",
    json!({"title":"Feature","source_branch":"feature","target_branch":"main","squash":true}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  assert_eq!(created.1["data"]["iid"], 1);
  assert_eq!(created.1["data"]["squash"], true);
  assert_eq!(created.1["data"]["draft"], false);
  let head = s
    .git
    .resolve_revision(repo, "feature", "main")
    .await
    .unwrap();
  assert_eq!(created.1["data"]["head_sha"], head.as_str());
  s.db
    .insert(
      "project_branch_protections",
      fields(json!({
          "project_id":20,
          "branch_name":"main",
          "rule_type":"exact",
          "push_access_level":"no_one",
          "merge_access_level":"maintainer",
          "require_merge_request":1,
          "require_pipeline_success":1,
          "allow_force_push":0,
          "allow_delete":0,
      })),
    )
    .await
    .unwrap();
  let checks = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/checks",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(checks.0, StatusCode::OK);
  assert_eq!(checks.1["data"]["mergeable"], false);
  assert_eq!(checks.1["data"]["status"], "missing");
  let sha = checks.1["data"]["source_commit_sha"].as_str().unwrap();
  // A green pipeline for another SHA must never satisfy this merge request.
  let mut pipeline_id = 0;
  for (iid, sha) in [(1, "old-commit"), (2, sha)] {
    let pipeline = s
      .db
      .insert(
        "project_pipelines",
        fields(json!({
            "project_id":20,
            "iid":iid,
            "name":"Test",
            "source":"push",
            "ref_name":"feature",
            "commit_sha":sha,
            "status":"succeeded",
            "config_source":".gity-ci.yml",
            "config_content":"test: {}",
            "started_at":now(),
            "finished_at":now(),
        })),
      )
      .await
      .unwrap();
    pipeline_id = number(&pipeline, "id");
  }
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/merge-requests/1/approve",
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );
  let self_approved = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/checks",
    "alice",
    json!({}),
  )
  .await
  .1;
  assert_eq!(self_approved["data"]["approval_count"], 0);
  assert_eq!(self_approved["data"]["mergeable"], false);
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/merge-requests/1/merge",
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/merge-requests/1/approve",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );
  // Duplicate approval is idempotent; it cannot inflate the threshold count.
  request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/approve",
    "bob",
    json!({}),
  )
  .await;
  // A developer cannot satisfy checks with an easier inline pipeline at the
  // correct SHA while the repository requires a different committed YAML.
  let untrusted = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/checks",
    "alice",
    json!({}),
  )
  .await
  .1;
  assert_eq!(untrusted["data"]["mergeable"], false);
  assert_eq!(
    untrusted["data"]["blockers"][0]["code"],
    "pipeline_config_mismatch"
  );
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/merge-requests/1/merge",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
  s.db
    .update(
      "project_pipelines",
      pipeline_id,
      fields(json!({"config_content":"test:\n  script: echo ok\n"})),
    )
    .await
    .unwrap();
  let checked = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/checks",
    "alice",
    json!({}),
  )
  .await
  .1;
  assert_eq!(checked["data"]["approval_count"], 1);
  assert_eq!(checked["data"]["mergeable"], true);
  let merged = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/merge",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(merged.1["data"]["state"], "merged");
  let tip = s.git.resolve_revision(repo, "main", "main").await.unwrap();
  assert_eq!(merged.1["data"]["merge_commit_sha"], tip.as_str());
  assert_eq!(merged.1["data"]["merged_by_user_id"], "2");
  assert!(merged.1["data"]["merged_at"].is_string());
  assert!(merged.1["data"]["closed_at"].is_null());
  assert!(
    s.git
      .file_exists(repo, "main", "main", ".gity-ci.yml")
      .await
      .unwrap()
  );
  assert_eq!(
    request(
      &s,
      "PATCH",
      "/projects/20/merge-requests/1",
      "alice",
      json!({"state":"opened"})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/merge-requests/1/merge",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
}
