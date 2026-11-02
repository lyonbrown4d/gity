//! Comment editing, deletion and review notifications over the HTTP surface.
use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

async fn new_issue(s: &AppState, title: &str) -> String {
  let (status, body) = request(
    s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({ "title": title }),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{body}");
  body["data"]["id"].as_str().unwrap().into()
}

async fn new_issue_comment(s: &AppState, iid: &str, user: &str, text: &str) -> String {
  let path = format!("/projects/20/issues/{iid}/comments");
  let (status, body) = request(s, "POST", &path, user, json!({ "body": text })).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  body["data"]["id"].as_str().unwrap().into()
}

async fn new_merge_comment(s: &AppState, iid: i64, user: &str, text: &str) -> String {
  let path = format!("/projects/20/merge-requests/{iid}/comments");
  let (status, body) = request(s, "POST", &path, user, json!({ "content": text })).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  let comments = body["data"]["comments"].as_array().unwrap();
  comments.last().unwrap()["id"].as_str().unwrap().into()
}

/// Merge request rows are inserted directly; branch validation is not the
/// subject of these tests.
async fn merge_request(s: &AppState, iid: i64, author: i64) {
  s.db
    .insert(
      "project_merge_requests",
      fields(json!({
          "project_id":20,"iid":iid,"author_user_id":author,
          "title":format!("Merge request {iid}"),"state":"opened",
          "source_branch":format!("feature-{iid}"),"target_branch":"main",
      })),
    )
    .await
    .unwrap();
}

async fn reporter(s: &AppState, user: i64) {
  s.db
    .insert(
      "project_members",
      fields(json!({ "project_id":20,"user_id":user,"role":"reporter" })),
    )
    .await
    .unwrap();
}

async fn todos(s: &AppState, user: &str, kind: &str) -> Vec<Value> {
  let (status, body) = request(s, "GET", "/todos?state=pending", user, json!({})).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  body["data"]
    .as_array()
    .unwrap()
    .iter()
    .filter(|todo| todo["kind"] == kind)
    .cloned()
    .collect()
}

#[tokio::test]
async fn issue_comment_edits_belong_to_the_author_or_a_maintainer() {
  let (_dir, s) = fixture().await;
  let issue = new_issue(&s, "Comment permissions").await;
  let path = format!("/projects/20/issues/{issue}/comments");
  let comment = new_issue_comment(&s, &issue, "bob", "Original").await;
  let url = format!("{path}/{comment}");
  let (status, body) = request(&s, "PATCH", &url, "bob", json!({ "content": "Edited" })).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(body["data"]["content"], "Edited");
  let (status, body) = request(&s, "PATCH", &url, "alice", json!({ "body": "Moderated" })).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(body["data"]["content"], "Moderated");

  reporter(&s, 3).await;
  let mine = new_issue_comment(&s, &issue, "outsider", "Mine").await;
  let own = format!("{path}/{mine}");
  let (status, body) = request(
    &s,
    "PATCH",
    &own,
    "outsider",
    json!({ "body": "Mine again" }),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(body["data"]["content"], "Mine again");
  assert_eq!(
    request(&s, "PATCH", &url, "outsider", json!({ "body": "Hijack" }))
      .await
      .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(&s, "DELETE", &url, "outsider", json!({})).await.0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(&s, "PATCH", &own, "outsider", json!({ "body": " " }))
      .await
      .0,
    StatusCode::BAD_REQUEST
  );

  let other = new_issue(&s, "Other issue").await;
  let misplaced = format!("/projects/20/issues/{other}/comments/{mine}");
  assert_eq!(
    request(
      &s,
      "PATCH",
      &misplaced,
      "outsider",
      json!({ "body": "Wrong issue" })
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  let (status, body) = request(&s, "DELETE", &own, "outsider", json!({})).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(body["data"]["ok"], true);
  let listed = request(&s, "GET", &path, "alice", json!({})).await.1;
  assert!(
    !listed["data"]
      .as_array()
      .unwrap()
      .iter()
      .any(|row| row["id"] == mine.as_str())
  );
}

#[tokio::test]
async fn merge_request_comments_notify_the_author_and_stay_editable() {
  let (_dir, s) = fixture().await;
  merge_request(&s, 1, 1).await;
  reporter(&s, 3).await;
  let comment = new_merge_comment(&s, 1, "bob", "Please split this").await;
  let notified = todos(&s, "alice", "merge_request_commented").await;
  assert_eq!(notified.len(), 1);
  assert_eq!(notified[0]["target_id"], comment.as_str());
  assert!(
    notified[0]["action_url"]
      .as_str()
      .unwrap()
      .ends_with("?tab=merge-requests")
  );
  new_merge_comment(&s, 1, "alice", "Acknowledged").await;
  assert_eq!(todos(&s, "alice", "merge_request_commented").await.len(), 1);
  assert!(todos(&s, "bob", "merge_request_commented").await.is_empty());

  let url = format!("/projects/20/merge-requests/1/comments/{comment}");
  assert_eq!(
    request(
      &s,
      "PATCH",
      &url,
      "bob",
      json!({ "content": "Split applied" })
    )
    .await
    .0,
    StatusCode::OK
  );
  assert_eq!(
    request(&s, "PATCH", &url, "alice", json!({ "body": "Thanks" }))
      .await
      .0,
    StatusCode::OK
  );
  assert_eq!(
    request(&s, "PATCH", &url, "outsider", json!({ "body": "Hijack" }))
      .await
      .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(&s, "DELETE", &url, "outsider", json!({})).await.0,
    StatusCode::FORBIDDEN
  );
  let own = new_merge_comment(&s, 1, "outsider", "Unrelated").await;
  let (status, body) = request(
    &s,
    "DELETE",
    &format!("/projects/20/merge-requests/1/comments/{own}"),
    "outsider",
    json!({}),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(body["data"]["ok"], true);
}

#[tokio::test]
async fn approvals_and_review_requests_reach_the_author_and_the_assignee() {
  let (_dir, s) = fixture().await;
  merge_request(&s, 1, 1).await;
  let approve = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/approve",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(approve.0, StatusCode::OK, "{}", approve.1);
  assert_eq!(todos(&s, "alice", "merge_request_approved").await.len(), 1);
  assert!(todos(&s, "bob", "merge_request_approved").await.is_empty());
  let unapprove = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/unapprove",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(unapprove.0, StatusCode::OK, "{}", unapprove.1);
  assert_eq!(
    todos(&s, "alice", "merge_request_unapproved").await.len(),
    1
  );

  let own = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/approve",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(own.0, StatusCode::OK, "{}", own.1);
  assert_eq!(todos(&s, "alice", "merge_request_approved").await.len(), 1);

  let reviewers = request(
    &s,
    "PATCH",
    "/projects/20/merge-requests/1/reviewers",
    "alice",
    json!({ "user_ids": [1, 2] }),
  )
  .await;
  assert_eq!(reviewers.0, StatusCode::OK, "{}", reviewers.1);
  assert_eq!(
    todos(&s, "bob", "merge_request_review_requested")
      .await
      .len(),
    1
  );
  assert!(
    todos(&s, "alice", "merge_request_review_requested")
      .await
      .is_empty()
  );
  let assignees = request(
    &s,
    "PATCH",
    "/projects/20/merge-requests/1/assignees",
    "alice",
    json!({ "user_ids": [2] }),
  )
  .await;
  assert_eq!(assignees.0, StatusCode::OK, "{}", assignees.1);
  let assigned = todos(&s, "bob", "merge_request_assigned").await;
  assert_eq!(assigned.len(), 1);
  assert!(assigned[0]["title"].as_str().unwrap().contains("!1"));
}
