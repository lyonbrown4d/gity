use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
use gity_entities::{project_audit_events, project_subscriptions};
use sea_orm::{ActiveModelTrait, Set};

async fn confidential_issue(s: &AppState) -> (String, i64) {
  let created = request(
    s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"secret list title","description":"secret description","confidential":true}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK, "{}", created.1);
  let iid = created.1["data"]["id"].as_str().unwrap().to_owned();
  let issue = s
    .collaboration
    .issue(20, iid.parse().unwrap())
    .await
    .unwrap();
  let comment = request(
    s,
    "POST",
    &format!("/projects/20/issues/{iid}/comments"),
    "alice",
    json!({"body":"secret comment summary"}),
  )
  .await;
  assert_eq!(comment.0, StatusCode::OK, "{}", comment.1);
  (
    comment.1["data"]["id"].as_str().unwrap().to_owned(),
    issue.id,
  )
}

async fn add_saved_rows(s: &AppState, users: &[i64], comment_id: &str, issue_id: i64) {
  let now = chrono::Utc::now().naive_utc();
  project_audit_events::ActiveModel {
    id: Set(gity_core::new_id()),
    project_id: Set(20),
    organization_id: Set(10),
    event_name: Set("project.issues.comment".to_owned()),
    action: Set("comment_issue".to_owned()),
    actor_user_id: Set(1),
    target_type: Set("issue".to_owned()),
    target_id: Set(issue_id.to_string()),
    summary: Set("secret activity summary".to_owned()),
    payload: Set("{}".to_owned()),
    created_at: Set(now),
  }
  .insert(&s.db.connection)
  .await
  .unwrap();

  for user in users {
    s.db
      .insert(
        "project_todos",
        fields(json!({
          "user_id":user,
          "organization_id":10,
          "project_id":20,
          "kind":"issue_commented",
          "state":"pending",
          "target_type":"issue_comment",
          "target_id":comment_id,
          "title":"secret todo title",
          "summary":"secret todo summary",
          "action_url":"/private/issue",
        })),
      )
      .await
      .unwrap();
    project_subscriptions::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(20),
      user_id: Set(*user),
      target_type: Set("issue".to_owned()),
      target_id: Set(issue_id),
      created_at: Set(now),
    }
    .insert(&s.db.connection)
    .await
    .unwrap();
  }
}

#[tokio::test]
async fn public_project_guest_and_outsider_lists_hide_confidential_targets() {
  let (_dir, s) = fixture().await;
  s.db
    .execute(
      "UPDATE projects SET visibility='public' WHERE id=20",
      vec![],
    )
    .await
    .unwrap();
  s.db
    .execute(
      "UPDATE project_members SET role='guest' WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  let (comment_id, issue_id) = confidential_issue(&s).await;
  add_saved_rows(&s, &[2, 3], &comment_id, issue_id).await;

  for user in ["bob", "outsider"] {
    let todos = request(&s, "GET", "/todos", user, json!({})).await;
    assert_eq!(todos.0, StatusCode::OK);
    assert!(todos.1["data"].as_array().unwrap().is_empty(), "{user}");

    let activity = request(&s, "GET", "/activity", user, json!({})).await;
    assert_eq!(activity.0, StatusCode::OK);
    assert!(
      activity.1["data"]["items"].as_array().unwrap().is_empty(),
      "{user}"
    );

    let subscriptions = request(&s, "GET", "/subscriptions", user, json!({})).await;
    assert_eq!(subscriptions.0, StatusCode::OK);
    assert!(
      subscriptions.1["data"].as_array().unwrap().is_empty(),
      "{user}"
    );
  }
}

#[tokio::test]
async fn removed_project_membership_hides_historical_todo_activity_and_subscription() {
  let (_dir, s) = fixture().await;
  let (comment_id, issue_id) = confidential_issue(&s).await;
  add_saved_rows(&s, &[2], &comment_id, issue_id).await;
  s.db
    .execute(
      "DELETE FROM project_members WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();

  let todos = request(&s, "GET", "/todos", "bob", json!({})).await;
  assert_eq!(todos.0, StatusCode::OK);
  assert!(todos.1["data"].as_array().unwrap().is_empty());
  let activity = request(&s, "GET", "/activity", "bob", json!({})).await;
  assert_eq!(activity.0, StatusCode::OK);
  assert!(activity.1["data"]["items"].as_array().unwrap().is_empty());
  let subscriptions = request(&s, "GET", "/subscriptions", "bob", json!({})).await;
  assert_eq!(subscriptions.0, StatusCode::OK);
  assert!(subscriptions.1["data"].as_array().unwrap().is_empty());
}
