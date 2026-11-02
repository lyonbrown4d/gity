use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
#[tokio::test]
async fn merge_request_patch_tracks_draft_squash_and_close_state() {
  let (_dir, s) = fixture().await;
  s.db
    .insert(
      "project_merge_requests",
      fields(json!({
          "project_id":20,"iid":1,"author_user_id":1,"title":"Draft",
          "state":"opened","source_branch":"feature","target_branch":"main",
      })),
    )
    .await
    .unwrap();
  s.db
    .insert(
      "project_milestones",
      fields(json!({"id":777,"project_id":20,"iid":1,"title":"v1","state":"active"})),
    )
    .await
    .unwrap();
  let patched = request(
    &s,
    "PATCH",
    "/projects/20/merge-requests/1",
    "alice",
    json!({"draft":true,"squash":true,"milestone_id":777}),
  )
  .await;
  assert_eq!(patched.0, StatusCode::OK);
  assert_eq!(patched.1["data"]["draft"], true);
  assert_eq!(patched.1["data"]["squash"], true);
  assert_eq!(patched.1["data"]["milestone_id"], "777");
  assert!(patched.1["data"]["closed_at"].is_null());
  assert!(patched.1["data"]["merged_at"].is_null());
  let closed = request(
    &s,
    "PATCH",
    "/projects/20/merge-requests/1",
    "alice",
    json!({"state":"closed","milestone_id":null}),
  )
  .await;
  assert_eq!(closed.1["data"]["state"], "closed");
  assert!(closed.1["data"]["closed_at"].is_string());
  assert!(closed.1["data"]["milestone_id"].is_null());
  let reopened = request(
    &s,
    "PATCH",
    "/projects/20/merge-requests/1",
    "alice",
    json!({"state":"opened"}),
  )
  .await;
  assert_eq!(reopened.0, StatusCode::OK);
  assert!(reopened.1["data"]["closed_at"].is_null());
}
#[tokio::test]
async fn merge_request_rejects_unknown_milestone() {
  let (_dir, s) = fixture().await;
  s.db
    .insert(
      "project_merge_requests",
      fields(json!({
          "project_id":20,"iid":1,"author_user_id":1,"title":"Draft",
          "state":"opened","source_branch":"feature","target_branch":"main",
      })),
    )
    .await
    .unwrap();
  let rejected = request(
    &s,
    "PATCH",
    "/projects/20/merge-requests/1",
    "alice",
    json!({"milestone_id":999}),
  )
  .await;
  assert_eq!(rejected.0, StatusCode::NOT_FOUND);
}
