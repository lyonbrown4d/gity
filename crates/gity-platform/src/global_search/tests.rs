use crate::AppState;
use crate::features::test_support::{fixture, request};
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

async fn search(s: &AppState, token: &str, query: &str) -> (StatusCode, Value) {
  let response = crate::router(s.clone())
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/search")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "query":query,
            "types":["issue"],
            "project_ids":["20"],
            "limit":10
          })
          .to_string(),
        ))
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

async fn prepare() -> (tempfile::TempDir, AppState) {
  let (dir, mut s) = fixture().await;
  Arc::make_mut(&mut s.settings).search.global.enabled = true;
  s.git.init_bare("20.git", "main").await.unwrap();
  (dir, s)
}

#[tokio::test]
async fn real_issue_projection_hides_newly_confidential_issue_before_reindex() {
  let (_dir, s) = prepare().await;
  let created = request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"projection needle","description":"projection body"}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK, "{}", created.1);
  crate::global_search::worker::reconcile(&s).await.unwrap();
  crate::global_search::worker::run_pending(&s).await.unwrap();
  let initial = search(&s, "alice", "projection needle").await;
  assert_eq!(initial.0, StatusCode::OK, "{}", initial.1);
  assert_eq!(
    initial.1["data"]["groups"][0]["items"]
      .as_array()
      .unwrap()
      .len(),
    1
  );
  s.db
    .execute(
      "UPDATE project_members SET role='guest' WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();

  let iid = created.1["data"]["id"].as_str().unwrap();
  let updated = request(
    &s,
    "PATCH",
    &format!("/projects/20/issues/{iid}"),
    "alice",
    json!({"confidential":true}),
  )
  .await;
  assert_eq!(updated.0, StatusCode::OK, "{}", updated.1);
  // The published index still says public. Hydration must apply current confidentiality.
  let stale = search(&s, "bob", "projection needle").await;
  assert_eq!(stale.0, StatusCode::OK, "{}", stale.1);
  assert!(
    stale.1["data"]["groups"][0]["items"]
      .as_array()
      .unwrap()
      .is_empty()
  );
  crate::global_search::worker::reconcile(&s).await.unwrap();
  crate::global_search::worker::run_pending(&s).await.unwrap();
  let rebuilt = search(&s, "bob", "projection needle").await;
  assert_eq!(rebuilt.0, StatusCode::OK, "{}", rebuilt.1);
  assert!(
    rebuilt.1["data"]["groups"][0]["items"]
      .as_array()
      .unwrap()
      .is_empty()
  );
}

#[tokio::test]
async fn global_issue_search_rejects_anonymous_and_revoked_project_membership() {
  let (_dir, s) = prepare().await;
  request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"revocation needle"}),
  )
  .await;
  crate::global_search::worker::reconcile(&s).await.unwrap();
  crate::global_search::worker::run_pending(&s).await.unwrap();
  assert_eq!(
    search(&s, "", "revocation needle").await.0,
    StatusCode::UNAUTHORIZED
  );
  assert_eq!(
    search(&s, "bob", "revocation needle").await.0,
    StatusCode::OK
  );
  s.db
    .execute(
      "DELETE FROM project_members WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  let revoked = search(&s, "bob", "revocation needle").await;
  assert_eq!(revoked.0, StatusCode::OK, "{}", revoked.1);
  assert!(
    revoked.1["data"]["groups"][0]["items"]
      .as_array()
      .unwrap()
      .is_empty()
  );
}
