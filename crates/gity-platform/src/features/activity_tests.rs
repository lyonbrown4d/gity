use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
use gity_entities::{project_audit_events, project_issues};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

/// Audit rows carry no `updated_at` column, so they are inserted through the
/// same typed model the application writes them with.
async fn record(s: &AppState, id: i64, project: i64, name: &str, actor: i64, at: &str) -> i64 {
  let target_id = ensure_issue_target(s, project).await;
  project_audit_events::ActiveModel {
    id: Set(id),
    project_id: Set(project),
    organization_id: Set(10),
    event_name: Set(name.to_owned()),
    action: Set("comment_issue".to_owned()),
    actor_user_id: Set(actor),
    target_type: Set("issue".to_owned()),
    target_id: Set(target_id.to_string()),
    summary: Set(format!("{name} recorded")),
    payload: Set("{}".to_owned()),
    created_at: Set(
      chrono::NaiveDateTime::parse_from_str(at, "%Y-%m-%d %H:%M:%S").expect("fixture timestamp"),
    ),
  }
  .insert(&s.db.connection)
  .await
  .unwrap();
  target_id
}

async fn ensure_issue_target(s: &AppState, project: i64) -> i64 {
  if let Some(issue) = project_issues::Entity::find()
    .filter(project_issues::Column::ProjectId.eq(project))
    .one(&s.db.connection)
    .await
    .unwrap()
  {
    return issue.id;
  }
  let now = chrono::Utc::now().naive_utc();
  let issue = project_issues::ActiveModel {
    id: Set(gity_core::new_id()),
    project_id: Set(project),
    iid: Set(1),
    author_user_id: Set(1),
    title: Set("Activity test issue".to_owned()),
    description: Set(None),
    state: Set("opened".to_owned()),
    milestone_id: Set(None),
    due_date: Set(None),
    weight: Set(None),
    confidential: Set(0),
    closed_by_user_id: Set(None),
    closed_at: Set(None),
    updated_by_user_id: Set(None),
    issue_type: Set("issue".to_owned()),
    created_at: Set(now),
    updated_at: Set(now),
  }
  .insert(&s.db.connection)
  .await
  .unwrap();
  issue.id
}

fn items(body: &Value) -> &Vec<Value> {
  body["data"]["items"].as_array().expect("activity items")
}

#[tokio::test]
async fn activity_feed_scopes_events_to_readable_projects() {
  let (_dir, s) = fixture().await;
  record(
    &s,
    9001,
    20,
    "project.issues.comment",
    1,
    "2026-01-01 09:00:00",
  )
  .await;
  record(
    &s,
    9002,
    20,
    "project.repository.push",
    1,
    "2026-01-02 09:00:00",
  )
  .await;
  let project30_issue = record(
    &s,
    9003,
    30,
    "project.merge_requests.merge",
    2,
    "2026-01-03 09:00:00",
  )
  .await;
  let (status, body) = request(&s, "GET", "/activity", "alice", json!({})).await;
  assert_eq!(status, StatusCode::OK);
  let feed = items(&body);
  assert_eq!(feed.len(), 3);
  assert_eq!(feed[0]["id"], "9003");
  assert_eq!(feed[0]["event_name"], "project.merge_requests.merge");
  assert_eq!(feed[2]["id"], "9001");
  assert_eq!(feed[0]["actor_user_id"], "2");
  assert_eq!(feed[0]["actor_name"], "bob");
  assert_eq!(feed[0]["project_id"], "30");
  assert_eq!(feed[0]["project_name"], "Project 30");
  assert_eq!(feed[0]["organization_id"], "10");
  assert_eq!(feed[0]["project_path"], "team/p30");
  assert_eq!(feed[0]["target_type"], "issue");
  assert_eq!(feed[0]["target_id"], project30_issue.to_string());
  assert_eq!(feed[0]["action"], "comment_issue");
  assert_eq!(feed[0]["summary"], "project.merge_requests.merge recorded");
  assert!(
    feed[0]["created_at"]
      .as_str()
      .expect("timestamp")
      .starts_with("2026-01-03T")
  );
  let (status, body) = request(&s, "GET", "/activity", "outsider", json!({})).await;
  assert_eq!(status, StatusCode::OK);
  assert!(items(&body).is_empty(), "private projects must stay hidden");
  assert_eq!(
    request(&s, "GET", "/activity", "", json!({})).await.0,
    StatusCode::UNAUTHORIZED
  );
}

#[tokio::test]
async fn activity_feed_includes_public_and_organization_projects() {
  let (_dir, s) = fixture().await;
  s.db
    .insert(
      "projects",
      fields(json!({"id":40,"organization_id":10,"name":"Public 40","path_key":"p40","full_path":"team/p40","visibility":"public","default_branch":"main","status":"active"})),
    )
    .await
    .unwrap();
  record(
    &s,
    9001,
    20,
    "project.issues.create",
    1,
    "2026-02-01 09:00:00",
  )
  .await;
  record(
    &s,
    9002,
    40,
    "project.issues.create",
    1,
    "2026-02-02 09:00:00",
  )
  .await;
  assert_eq!(
    items(&request(&s, "GET", "/activity", "alice", json!({})).await.1).len(),
    2
  );
  let public = request(&s, "GET", "/activity", "outsider", json!({})).await;
  assert_eq!(items(&public.1).len(), 1);
  assert_eq!(items(&public.1)[0]["project_id"], "40");
  assert_eq!(items(&public.1)[0]["project_name"], "Public 40");
  assert_eq!(items(&public.1)[0]["project_path"], "team/p40");
  assert_eq!(items(&public.1)[0]["organization_id"], "10");
  s.db
    .insert(
      "organization_members",
      fields(json!({"organization_id":10,"user_id":3,"role":"guest"})),
    )
    .await
    .unwrap();
  record(
    &s,
    9003,
    30,
    "project.issues.closed",
    2,
    "2026-02-03 09:00:00",
  )
  .await;
  let organization = request(&s, "GET", "/activity", "outsider", json!({}))
    .await
    .1;
  assert_eq!(
    items(&organization).len(),
    3,
    "organization membership exposes the private projects it owns"
  );
  assert_eq!(items(&organization)[0]["project_id"], "30");
}

#[tokio::test]
async fn activity_feed_limits_and_orders_newest_first() {
  let (_dir, s) = fixture().await;
  for offset in 1..=5 {
    record(
      &s,
      9100 + offset,
      20,
      "project.issues.comment",
      1,
      &format!("2026-03-0{offset} 09:00:00"),
    )
    .await;
  }
  let limited = request(&s, "GET", "/activity?limit=2", "alice", json!({})).await;
  assert_eq!(items(&limited.1).len(), 2);
  assert_eq!(items(&limited.1)[0]["id"], "9105");
  assert_eq!(items(&limited.1)[1]["id"], "9104");
  for query in ["?limit=500", "?limit=0", "?limit=abc"] {
    let body = request(&s, "GET", &format!("/activity{query}"), "alice", json!({}))
      .await
      .1;
    assert_eq!(
      items(&body).len(),
      5,
      "{query} must fall back to a bounded default"
    );
  }
  let (status, _) = request(&s, "GET", "/activity?limit=1", "bob", json!({})).await;
  assert_eq!(status, StatusCode::OK);
  assert_eq!(
    items(
      &request(&s, "GET", "/activity?limit=1", "bob", json!({}))
        .await
        .1
    )
    .len(),
    1
  );
}
