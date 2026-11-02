use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

fn kind<'a>(todos: &'a Value, kind: &str) -> Vec<&'a Value> {
  todos["data"]
    .as_array()
    .unwrap()
    .iter()
    .filter(|todo| todo["kind"] == kind)
    .collect()
}

async fn preference(s: &AppState, user: &str, body: Value) -> (StatusCode, Value) {
  request(s, "PUT", "/users/me/notification-preferences", user, body).await
}

#[tokio::test]
async fn comment_notifications_reach_subscribers_unless_muted() {
  let (_dir, s) = fixture().await;
  // A third member comments so the subscriber is not the comment author.
  s.db
    .insert(
      "project_members",
      fields(json!({"project_id":20,"user_id":3,"role":"maintainer"})),
    )
    .await
    .unwrap();
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/issues",
      "alice",
      json!({"title":"Notify me"})
    )
    .await
    .0,
    StatusCode::OK
  );
  assert_eq!(
    request(
      &s,
      "PUT",
      "/projects/20/issues/1/subscription",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );

  let commented = request(
    &s,
    "POST",
    "/projects/20/issues/1/comments",
    "outsider",
    json!({"body":"first"}),
  )
  .await;
  assert_eq!(commented.0, StatusCode::OK);
  let subscriber = request(&s, "GET", "/todos?state=pending", "bob", json!({}))
    .await
    .1;
  assert_eq!(kind(&subscriber, "issue_commented").len(), 1);
  let author = request(&s, "GET", "/todos?state=pending", "alice", json!({}))
    .await
    .1;
  assert_eq!(kind(&author, "issue_commented").len(), 1);
  // The comment author never receives their own comment notification.
  let commenter = request(&s, "GET", "/todos?state=pending", "outsider", json!({}))
    .await
    .1;
  assert_eq!(kind(&commenter, "issue_commented").len(), 0);

  let muted = preference(&s, "bob", json!({"kind":"issue_commented","enabled":false})).await;
  assert_eq!(muted.0, StatusCode::OK);
  assert_eq!(muted.1["data"]["enabled"], false);
  assert_eq!(muted.1["data"]["explicit"], true);
  let preferences = request(
    &s,
    "GET",
    "/users/me/notification-preferences",
    "bob",
    json!({}),
  )
  .await
  .1;
  let entry = |name: &str| {
    preferences["data"]
      .as_array()
      .unwrap()
      .iter()
      .find(|value| value["kind"] == name)
      .unwrap()
      .clone()
  };
  assert_eq!(entry("issue_commented")["enabled"], false);
  assert_eq!(entry("issue_commented")["explicit"], true);
  // Untouched kinds keep the enabled default.
  assert_eq!(entry("issue_created")["enabled"], true);
  assert_eq!(entry("issue_created")["explicit"], false);

  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/issues/1/comments",
      "outsider",
      json!({"body":"second"})
    )
    .await
    .0,
    StatusCode::OK
  );
  let after = request(&s, "GET", "/todos?state=pending", "bob", json!({}))
    .await
    .1;
  assert_eq!(kind(&after, "issue_commented").len(), 1);
  let author_after = request(&s, "GET", "/todos?state=pending", "alice", json!({}))
    .await
    .1;
  assert_eq!(kind(&author_after, "issue_commented").len(), 2);
}

#[tokio::test]
async fn preference_updates_validate_kind_and_authentication() {
  let (_dir, s) = fixture().await;
  assert_eq!(
    preference(&s, "bob", json!({"kind":"Issue Commented","enabled":true}))
      .await
      .0,
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    preference(&s, "bob", json!({"kind":"issue_commented"}))
      .await
      .0,
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    request(
      &s,
      "GET",
      "/users/me/notification-preferences",
      "",
      json!({})
    )
    .await
    .0,
    StatusCode::UNAUTHORIZED
  );
  let stored = preference(
    &s,
    "bob",
    json!({"kind":"merge_request_commented","enabled":true}),
  )
  .await;
  assert_eq!(stored.0, StatusCode::OK);
  assert_eq!(stored.1["data"]["explicit"], true);
  assert!(stored.1["data"]["updated_at"].is_string());
}
