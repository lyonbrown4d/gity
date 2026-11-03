use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
fn file(branch: &str, path: &str, content: &str) -> gity_git::CreateFileCommitInput {
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
async fn codeowner_rules_use_target_branch_last_matching_owner_and_ignore_author_approval() {
  let (_dir, s) = fixture().await;
  let repo = "20.git";
  s.git.init_bare(repo, "main").await.unwrap();
  s.git
    .create_file_commit(repo, file("main", "CODEOWNERS", "* @alice\nsrc/** @bob\n"))
    .await
    .unwrap();
  s.git.create_branch(repo, "feature", "main").await.unwrap();
  s.git
    .create_file_commit(repo, file("feature", "src/main.rs", "fn main() {}\n"))
    .await
    .unwrap();
  let rule = request(
    &s,
    "POST",
    "/projects/20/merge-request-approval-rules",
    "alice",
    json!({
        "name":"Code owners","target_branch":"main","approvals_required":1,
        "eligible_user_ids":[1],"code_owner":true
    }),
  )
  .await;
  assert_eq!(rule.0, StatusCode::OK);
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/merge-requests",
      "alice",
      json!({
          "title":"Source change","source_branch":"feature","target_branch":"main"
      })
    )
    .await
    .0,
    StatusCode::OK
  );
  request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/approve",
    "alice",
    json!({}),
  )
  .await;
  let blocked = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/checks",
    "alice",
    json!({}),
  )
  .await
  .1;
  assert_eq!(blocked["data"]["mergeable"], false);
  assert_eq!(
    blocked["data"]["approval_rules"][0]["eligible_user_ids"],
    json!(["2"])
  );
  request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/approve",
    "bob",
    json!({}),
  )
  .await;
  let passed = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/checks",
    "alice",
    json!({}),
  )
  .await
  .1;
  assert_eq!(passed["data"]["mergeable"], true);
}
