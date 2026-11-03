use crate::{AppState, features::test_support::fixture};
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

async fn prepare() -> (tempfile::TempDir, AppState) {
  let (dir, mut state) = fixture().await;
  Arc::make_mut(&mut state.settings).search.global.enabled = true;
  state.git.init_bare("20.git", "main").await.unwrap();
  state.git.init_bare("30.git", "main").await.unwrap();
  (dir, state)
}

async fn search(
  state: &AppState,
  token: &str,
  query: &str,
  types: &[&str],
  projects: &[&str],
  limit: u16,
  cursor: Option<&str>,
) -> (StatusCode, Value) {
  let mut input = json!({
    "query":query,
    "types":types,
    "project_ids":projects,
    "limit":limit,
  });
  if let Some(cursor) = cursor {
    input["cursor"] = json!(cursor);
  }
  let response = crate::router(state.clone())
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/search")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(input.to_string()))
        .unwrap(),
    )
    .await
    .unwrap();
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
  (
    status,
    serde_json::from_slice(&bytes).unwrap_or(Value::Null),
  )
}

async fn commit_file(
  state: &AppState,
  project: i64,
  path: &str,
  content: &str,
  update_existing: bool,
) -> String {
  let expected_head_sha = if update_existing {
    Some(
      state
        .git
        .resolve_revision(&format!("{project}.git"), "main", "")
        .await
        .unwrap(),
    )
  } else {
    None
  };
  state
    .git
    .create_file_commit(
      &format!("{project}.git"),
      serde_json::from_value(json!({
        "expected_head_sha":expected_head_sha,"update_existing":update_existing,
        "branch_name":"main","file_path":path,"content":content,
        "message":format!("update {path}"),"author_name":"Owner","author_email":"owner@example.test"
      }))
      .unwrap(),
    )
    .await
    .unwrap()
}

async fn index(state: &AppState) {
  crate::global_search::worker::reconcile(state)
    .await
    .unwrap();
  crate::global_search::worker::run_pending(state)
    .await
    .unwrap();
}

fn items(body: &Value) -> &Vec<Value> {
  body["data"]["groups"][0]["items"]
    .as_array()
    .expect("search result group")
}

#[tokio::test]
async fn code_projection_returns_live_path_line_and_commit_scoped_to_project() {
  let (_dir, state) = prepare().await;
  let commit20 = commit_file(
    &state,
    20,
    "src/match.rs",
    &format!(
      "{}pub fn start() {{\n  cursor_probe();\n}}\n",
      "\n".repeat(62)
    ),
    false,
  )
  .await;
  commit_file(
    &state,
    30,
    "src/other.rs",
    "pub fn other() {\n  cursor_probe();\n}\n",
    false,
  )
  .await;
  index(&state).await;

  let (status, response) = search(
    &state,
    "alice",
    "cursor_probe();\n}",
    &["code"],
    &["20"],
    10,
    None,
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{response}");
  let code = items(&response);
  assert_eq!(code.len(), 1);
  assert_eq!(code[0]["project_id"], "20");
  assert_eq!(code[0]["path"], "src/match.rs");
  assert_eq!(code[0]["line"], 64);
  assert_eq!(code[0]["commit_sha"], commit20);
}

async fn issue_cursor(state: &AppState, user: &str) -> String {
  for title in ["cursor issue alpha", "cursor issue beta"] {
    let created = crate::features::test_support::request(
      state,
      "POST",
      "/projects/20/issues",
      "alice",
      json!({"title":title,"description":"cursor details"}),
    )
    .await;
    assert_eq!(created.0, StatusCode::OK, "{}", created.1);
  }
  index(state).await;
  let (status, response) = search(state, user, "cursor issue", &["issue"], &["20"], 1, None).await;
  assert_eq!(status, StatusCode::OK, "{response}");
  assert_eq!(items(&response).len(), 1);
  response["data"]["next_cursor"]
    .as_str()
    .expect("more issue results create a cursor")
    .to_owned()
}

#[tokio::test]
async fn issue_cursor_rechecks_membership_and_returns_no_old_content() {
  let (_dir, state) = prepare().await;
  let cursor = issue_cursor(&state, "bob").await;
  state
    .db
    .execute(
      "DELETE FROM project_members WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();

  let (status, response) = search(
    &state,
    "bob",
    "cursor issue",
    &["issue"],
    &["20"],
    1,
    Some(&cursor),
  )
  .await;
  assert_eq!(status, StatusCode::CONFLICT, "{response}");
  assert!(!response.to_string().contains("cursor issue alpha"));
  assert!(!response.to_string().contains("cursor issue beta"));
}

#[tokio::test]
async fn revoked_personal_token_cannot_continue_an_issue_cursor() {
  let (_dir, state) = prepare().await;
  let cursor = issue_cursor(&state, "alice").await;
  state
    .db
    .execute("DELETE FROM user_access_tokens WHERE user_id=1", vec![])
    .await
    .unwrap();

  let (status, response) = search(
    &state,
    "alice",
    "cursor issue",
    &["issue"],
    &["20"],
    1,
    Some(&cursor),
  )
  .await;
  assert_eq!(status, StatusCode::UNAUTHORIZED, "{response}");
  assert!(!response.to_string().contains("cursor issue alpha"));
  assert!(!response.to_string().contains("cursor issue beta"));
}

#[tokio::test]
async fn stale_issue_documents_and_code_from_moved_default_ref_do_not_match() {
  let (_dir, state) = prepare().await;
  let created = crate::features::test_support::request(
    &state,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"old_issue_phrase"}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK, "{}", created.1);
  let old_commit = commit_file(
    &state,
    20,
    "src/stale.rs",
    "fn stale() { old_ref_phrase(); }\n",
    false,
  )
  .await;
  index(&state).await;

  let iid = created.1["data"]["id"].as_str().unwrap();
  let updated = crate::features::test_support::request(
    &state,
    "PATCH",
    &format!("/projects/20/issues/{iid}"),
    "alice",
    json!({"title":"new_issue_phrase"}),
  )
  .await;
  assert_eq!(updated.0, StatusCode::OK, "{}", updated.1);
  let new_commit = commit_file(
    &state,
    20,
    "src/fresh.rs",
    "fn fresh() { new_ref_phrase(); }\n",
    false,
  )
  .await;
  assert_ne!(old_commit, new_commit);

  let (issue_status, issue_response) = search(
    &state,
    "alice",
    "old_issue_phrase",
    &["issue"],
    &["20"],
    10,
    None,
  )
  .await;
  assert_eq!(issue_status, StatusCode::OK, "{issue_response}");
  assert!(items(&issue_response).is_empty());

  let (code_status, code_response) = search(
    &state,
    "alice",
    "old_ref_phrase",
    &["code"],
    &["20"],
    10,
    None,
  )
  .await;
  assert_eq!(code_status, StatusCode::OK, "{code_response}");
  assert!(items(&code_response).is_empty());
}
