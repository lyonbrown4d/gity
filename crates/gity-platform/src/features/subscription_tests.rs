use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

#[tokio::test]
async fn issue_subscription_lifecycle_is_personal_and_project_scoped() {
  let (_dir, s) = fixture().await;
  let created = request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"Subscribed issue"}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  assert_eq!(created.1["data"]["number"], 1);

  let status = request(
    &s,
    "GET",
    "/projects/20/issues/1/subscription",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(status.0, StatusCode::OK);
  assert_eq!(status.1["data"]["subscribed"], false);
  assert_eq!(status.1["data"]["target_type"], "issue");
  // The stored target is the application-generated issue ID, not the project iid.
  let target = status.1["data"]["target_id"].as_str().unwrap().to_owned();
  assert!(target.parse::<i64>().unwrap() > 1_000_000);

  let subscribed = request(
    &s,
    "PUT",
    "/projects/20/issues/1/subscription",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(subscribed.0, StatusCode::OK);
  assert_eq!(subscribed.1["data"]["subscribed"], true);
  // Repeated subscription requests stay idempotent.
  let repeated = request(
    &s,
    "PUT",
    "/projects/20/issues/1/subscription",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(repeated.0, StatusCode::OK);
  assert_eq!(repeated.1["data"]["target_id"], target);

  let mine = request(&s, "GET", "/subscriptions", "bob", json!({})).await;
  assert_eq!(mine.0, StatusCode::OK);
  assert_eq!(mine.1["data"].as_array().unwrap().len(), 1);
  assert_eq!(mine.1["data"][0]["target_id"], target);
  // Subscriptions are personal.
  assert_eq!(
    request(&s, "GET", "/subscriptions", "alice", json!({}))
      .await
      .1["data"]
      .as_array()
      .unwrap()
      .len(),
    0
  );

  let filtered = format!("/subscriptions?target_type=issue&target_id={target}");
  assert_eq!(
    request(&s, "GET", &filtered, "bob", json!({})).await.1["data"]
      .as_array()
      .unwrap()
      .len(),
    1
  );
  assert_eq!(
    request(
      &s,
      "GET",
      "/subscriptions?target_type=merge_request",
      "bob",
      json!({})
    )
    .await
    .1["data"]
      .as_array()
      .unwrap()
      .len(),
    0
  );
  assert_eq!(
    request(
      &s,
      "GET",
      "/subscriptions?target_type=branch",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );

  // Private project access still gates subscription reads and writes.
  for method in ["GET", "PUT"] {
    assert_eq!(
      request(
        &s,
        method,
        "/projects/20/issues/1/subscription",
        "outsider",
        json!({})
      )
      .await
      .0,
      StatusCode::FORBIDDEN
    );
  }
  assert_eq!(
    request(
      &s,
      "PUT",
      "/projects/20/issues/99/subscription",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );

  let removed = request(
    &s,
    "DELETE",
    "/projects/20/issues/1/subscription",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(removed.0, StatusCode::OK);
  assert_eq!(removed.1["data"]["subscribed"], false);
  // Unsubscribing twice is harmless.
  assert_eq!(
    request(
      &s,
      "DELETE",
      "/projects/20/issues/1/subscription",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );
  assert_eq!(
    request(
      &s,
      "GET",
      "/projects/20/issues/1/subscription",
      "bob",
      json!({})
    )
    .await
    .1["data"]["subscribed"],
    false
  );
}

#[tokio::test]
async fn merge_request_subscription_uses_the_stored_target() {
  let (_dir, s) = fixture().await;
  s.db
    .insert(
      "project_merge_requests",
      fields(json!({
          "project_id":20,"iid":1,"author_user_id":1,"title":"Subscription MR",
          "state":"opened","source_branch":"feature","target_branch":"main",
      })),
    )
    .await
    .unwrap();
  let target = request(&s, "GET", "/projects/20/merge-requests/1", "bob", json!({}))
    .await
    .1["data"]["id"]
    .as_str()
    .unwrap()
    .to_owned();
  let subscribed = request(
    &s,
    "PUT",
    "/projects/20/merge-requests/1/subscription",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(subscribed.0, StatusCode::OK);
  assert_eq!(subscribed.1["data"]["target_type"], "merge_request");
  assert_eq!(subscribed.1["data"]["target_id"], target);
  assert_eq!(
    request(
      &s,
      "GET",
      "/subscriptions?target_type=merge_request",
      "bob",
      json!({})
    )
    .await
    .1["data"]
      .as_array()
      .unwrap()
      .len(),
    1
  );
  assert_eq!(
    request(
      &s,
      "PUT",
      "/projects/20/merge-requests/99/subscription",
      "bob",
      json!({})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  let removed = request(
    &s,
    "DELETE",
    "/projects/20/merge-requests/1/subscription",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(removed.0, StatusCode::OK);
  assert_eq!(removed.1["data"]["subscribed"], false);
}
