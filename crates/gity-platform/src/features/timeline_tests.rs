use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

async fn comment(s: &AppState, iid: i64, body: &str) -> StatusCode {
  request(
    s,
    "POST",
    &format!("/projects/20/issues/{iid}/comments"),
    "bob",
    json!({"body": body}),
  )
  .await
  .0
}

async fn audit(s: &AppState, project: i64, target: &str, summary: &str) {
  crate::registry::record_event(
    s,
    crate::registry::AuditRecord {
      project,
      actor: 1,
      event: "project.issue.closed",
      action: "close_issue",
      target_type: "issue",
      target_id: target,
      summary,
      payload: json!({"state": "closed"}),
    },
  )
  .await
  .unwrap();
}

#[tokio::test]
async fn timeline_merges_comments_and_audit_events_in_ascending_order() {
  let (_dir, s) = fixture().await;
  let created = request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"Timeline ordering"}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  let iid = created.1["data"]["number"].as_i64().unwrap();
  assert_eq!(comment(&s, iid, "first comment").await, StatusCode::OK);
  let issue = s.collaboration.issue(20, iid).await.unwrap();
  let target = issue.id.to_string();
  assert_eq!(comment(&s, iid, "reply after close").await, StatusCode::OK);
  // An audit row for the same target id inside another project must not leak.
  audit(&s, 30, &target, "Wrong project").await;
  let closed = request(
    &s,
    "PATCH",
    &format!("/projects/20/issues/{iid}"),
    "alice",
    json!({"state":"closed"}),
  )
  .await;
  assert_eq!(closed.0, StatusCode::OK);
  assert_eq!(closed.1["data"]["status"], "closed");
  let (status, body) = request(
    &s,
    "GET",
    &format!("/projects/20/issues/{iid}/timeline"),
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let entries = body["data"].as_array().unwrap();
  let position = |predicate: &dyn Fn(&Value) -> bool| {
    entries.iter().position(predicate).expect("timeline entry")
  };
  let created = position(&|entry| entry["event_name"] == "project.issue.created");
  let first = position(&|entry| entry["body"] == "first comment");
  let reply = position(&|entry| entry["body"] == "reply after close");
  let closed = position(&|entry| entry["event_name"] == "project.issue.closed");
  assert!(created < first && first < reply && reply < closed);
  for entry in entries {
    if entry["type"] == "system" {
      assert!(entry["summary"].is_string());
      assert_eq!(entry["body"], Value::Null);
    }
    assert!(entry["id"].as_str().unwrap().parse::<i64>().unwrap() > 0);
    assert!(!entry["author_user_id"].as_str().unwrap().is_empty());
    assert!(entry["created_at"].as_str().unwrap().ends_with('Z'));
  }
  let stamps = entries
    .iter()
    .map(|entry| entry["created_at"].as_str().unwrap())
    .collect::<Vec<_>>();
  assert!(stamps.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[tokio::test]
async fn timeline_is_project_scoped_and_requires_read_access() {
  let (_dir, s) = fixture().await;
  let created = request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"Empty"}),
  )
  .await;
  let iid = created.1["data"]["number"].as_i64().unwrap();
  let path = format!("/projects/20/issues/{iid}/timeline");
  let timeline = request(&s, "GET", &path, "alice", json!({})).await;
  assert_eq!(timeline.0, StatusCode::OK);
  let entries = timeline.1["data"].as_array().unwrap();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0]["type"], "system");
  assert_eq!(entries[0]["event_name"], "project.issue.created");
  assert_eq!(
    request(&s, "GET", &path, "outsider", json!({})).await.0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(
      &s,
      "GET",
      "/projects/20/issues/99/timeline",
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
}

#[test]
fn timeline_route_is_documented_with_its_entry_schema() {
  let document = serde_json::to_value(crate::openapi::document()).unwrap();
  let operation = &document["paths"]["/api/v1/projects/{id}/issues/{iid}/timeline"]["get"];
  assert!(
    operation["operationId"]
      .as_str()
      .unwrap_or_default()
      .ends_with("timeline"),
    "timeline operation must be documented: {operation}"
  );
  let schema = &operation["responses"]["200"]["content"]["application/json"]["schema"];
  assert!(
    schema.to_string().contains("TimelineEntry"),
    "timeline response schema is missing: {schema}"
  );
}
