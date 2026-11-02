use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

#[tokio::test]
async fn milestone_crud_is_ordered_and_project_scoped() {
  let (_dir, s) = fixture().await;
  let created = request(
    &s,
    "POST",
    "/projects/20/milestones",
    "alice",
    json!({"title":"v1.0","description":"First release","start_date":"2026-01-01","due_date":"2026-02-01"}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  assert_eq!(created.1["data"]["iid"], 1);
  assert_eq!(created.1["data"]["state"], "active");
  assert_eq!(created.1["data"]["start_date"], "2026-01-01");
  assert_eq!(created.1["data"]["due_date"], "2026-02-01");
  let id = created.1["data"]["id"].as_str().unwrap().to_owned();
  assert!(id.parse::<i64>().unwrap() > 0);
  let second = request(
    &s,
    "POST",
    "/projects/20/milestones",
    "alice",
    json!({"title":"v2.0"}),
  )
  .await;
  assert_eq!(second.0, StatusCode::OK);
  assert_eq!(second.1["data"]["iid"], 2);
  assert_eq!(second.1["data"]["state"], "active");
  let list = request(&s, "GET", "/projects/20/milestones", "bob", json!({})).await;
  assert_eq!(list.0, StatusCode::OK);
  assert_eq!(list.1["data"].as_array().unwrap().len(), 2);
  assert_eq!(list.1["data"][0]["iid"], 1);
  assert_eq!(list.1["data"][1]["iid"], 2);
  let detail = request(&s, "GET", "/projects/20/milestones/1", "bob", json!({})).await;
  assert_eq!(detail.0, StatusCode::OK);
  assert_eq!(detail.1["data"]["id"], id);
  let updated = request(
    &s,
    "PATCH",
    "/projects/20/milestones/1",
    "alice",
    json!({"title":"1.0 GA","state":"closed"}),
  )
  .await;
  assert_eq!(updated.0, StatusCode::OK);
  assert_eq!(updated.1["data"]["title"], "1.0 GA");
  assert_eq!(updated.1["data"]["state"], "closed");
  assert_eq!(updated.1["data"]["description"], "First release");
  assert_eq!(
    request(
      &s,
      "PATCH",
      "/projects/20/milestones/1",
      "alice",
      json!({"state":"opened"})
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    request(
      &s,
      "PATCH",
      "/projects/20/milestones/1",
      "alice",
      json!({"due_date":"01/02/2026"})
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    request(&s, "GET", "/projects/30/milestones/1", "alice", json!({}))
      .await
      .0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/milestones",
      "outsider",
      json!({"title":"hidden"})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  let removed = request(
    &s,
    "DELETE",
    "/projects/20/milestones/2",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(removed.0, StatusCode::OK);
  assert_eq!(removed.1["data"]["status"], "deleted");
  assert_eq!(
    request(&s, "GET", "/projects/20/milestones", "alice", json!({}))
      .await
      .1["data"]
      .as_array()
      .unwrap()
      .len(),
    1
  );
}

#[tokio::test]
async fn issue_metadata_binds_milestone_and_records_closure() {
  let (_dir, s) = fixture().await;
  let milestone = request(
    &s,
    "POST",
    "/projects/20/milestones",
    "alice",
    json!({"title":"v1.0"}),
  )
  .await;
  let milestone_id = milestone.1["data"]["id"].as_str().unwrap().to_owned();
  let created = request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({
        "title":"Ship it",
        "milestone_id":milestone_id,
        "due_date":"2026-01-15",
        "weight":5,
        "confidential":true,
        "issue_type":"task",
    }),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  assert_eq!(created.1["data"]["milestone_id"], milestone_id);
  assert_eq!(created.1["data"]["due_date"], "2026-01-15");
  assert_eq!(created.1["data"]["weight"], 5);
  assert_eq!(created.1["data"]["confidential"], true);
  assert_eq!(created.1["data"]["issue_type"], "task");
  assert_eq!(created.1["data"]["closed_at"], Value::Null);
  let closed = request(
    &s,
    "PATCH",
    "/projects/20/issues/1",
    "alice",
    json!({"state":"closed"}),
  )
  .await;
  assert_eq!(closed.0, StatusCode::OK);
  assert_eq!(closed.1["data"]["status"], "closed");
  assert_eq!(closed.1["data"]["closed_by_user_id"], "1");
  assert!(closed.1["data"]["closed_at"].as_str().is_some());
  let reopened = request(
    &s,
    "PATCH",
    "/projects/20/issues/1",
    "alice",
    json!({"state":"opened"}),
  )
  .await;
  assert_eq!(reopened.1["data"]["status"], "open");
  assert_eq!(reopened.1["data"]["closed_at"], Value::Null);
  assert_eq!(reopened.1["data"]["closed_by_user_id"], Value::Null);
  let other = request(
    &s,
    "POST",
    "/projects/30/milestones",
    "alice",
    json!({"title":"other"}),
  )
  .await;
  let other_id = other.1["data"]["id"].clone();
  assert_eq!(
    request(
      &s,
      "PATCH",
      "/projects/20/issues/1",
      "alice",
      json!({"milestone_id":other_id})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  let detached = request(
    &s,
    "PATCH",
    "/projects/20/issues/1",
    "alice",
    json!({"milestone_id":null,"weight":null,"confidential":false}),
  )
  .await;
  assert_eq!(detached.0, StatusCode::OK);
  assert_eq!(detached.1["data"]["milestone_id"], Value::Null);
  assert_eq!(detached.1["data"]["weight"], Value::Null);
  assert_eq!(detached.1["data"]["confidential"], false);
  assert_eq!(detached.1["data"]["issue_type"], "task");
  let rebound = request(
    &s,
    "PATCH",
    "/projects/20/issues/1",
    "alice",
    json!({"milestone_id":milestone_id}),
  )
  .await;
  assert_eq!(rebound.1["data"]["milestone_id"], milestone_id);
  assert_eq!(
    request(
      &s,
      "DELETE",
      "/projects/20/milestones/1",
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );
  assert_eq!(
    request(&s, "GET", "/projects/20/issues/1", "alice", json!({}))
      .await
      .1["data"]["milestone_id"],
    Value::Null
  );
}
