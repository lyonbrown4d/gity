use super::*;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;
async fn call(router: &Router, method: &str, path: &str, token: &str, body: Value) -> Value {
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK, "{method} {path}");
  serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
}
#[tokio::test]
async fn dashboard_views_preserve_legacy_keys_roles_user_flags_and_clone_urls() {
  let (_dir, state) = crate::features::test_support::fixture().await;
  let router = crate::router(state.clone());
  let me = call(&router, "GET", "/api/v1/users/me", "alice", json!({})).await;
  assert_eq!(me["data"]["status"], "active");
  assert_eq!(me["data"]["is_super_admin"], false);
  let changed = call(
    &router,
    "PATCH",
    "/api/v1/users/me",
    "alice",
    json!({"username":"alice.renamed","email":"renamed@example.test","is_super_admin":true}),
  )
  .await;
  assert_eq!(changed["data"]["username"], "alice.renamed");
  let persisted = call(&router, "GET", "/api/v1/users/me", "alice", json!({})).await;
  assert_eq!(persisted["data"]["username"], "alice.renamed");
  assert_eq!(persisted["data"]["email"], "renamed@example.test");
  assert_eq!(persisted["data"]["is_super_admin"], false);
  for (name, expected) in [
    ("bad/name", StatusCode::BAD_REQUEST),
    ("", StatusCode::BAD_REQUEST),
    ("bob", StatusCode::CONFLICT),
  ] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .method("PATCH")
          .uri("/api/v1/users/me")
          .header("authorization", "Bearer alice")
          .header("content-type", "application/json")
          .body(Body::from(
            json!({"username":name,"email":"should-not-persist@example.test"}).to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), expected);
  }
  let unchanged = call(&router, "GET", "/api/v1/users/me", "alice", json!({})).await;
  assert_eq!(unchanged["data"]["username"], "alice.renamed");
  assert_eq!(unchanged["data"]["email"], "renamed@example.test");
  assert_eq!(
    state
      .db
      .one("SELECT full_path FROM organizations WHERE id=10", vec![])
      .await
      .unwrap()
      .unwrap()["full_path"],
    "team"
  );
  let protected = call(
    &router,
    "POST",
    "/api/v1/projects/20/repository/branches/main/protect",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(protected["data"]["require_merge_request"], true);
  assert_eq!(protected["data"]["allow_delete"], false);
  let rules = call(
    &router,
    "GET",
    "/api/v1/repos/20/branch-protections",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(rules["data"][0]["require_pipeline_success"], false);
  let created = call(
    &router,
    "POST",
    "/api/v1/orgs",
    "alice",
    json!({"name":"Custom Space","key":"custom-space"}),
  )
  .await;
  let org = created["data"]["id"].as_str().unwrap();
  assert_eq!(created["data"]["key"], "custom-space");
  assert_eq!(created["data"]["role"], "owner");
  state
    .db
    .insert(
      "organization_members",
      json!({"organization_id":org.parse::<i64>().unwrap(),"user_id":2,"role":"reporter"})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
  let reporter = call(
    &router,
    "GET",
    &format!("/api/v1/orgs/{org}"),
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(reporter["data"]["role"], "reporter");
  let project=call(&router,"POST","/api/v1/projects","alice",json!({"organization_id":org.parse::<i64>().unwrap(),"name":"Custom Project","key":"custom-project"})).await;
  assert_eq!(project["data"]["key"], "custom-project");
  assert_eq!(project["data"]["uuid"], project["data"]["id"]);
  assert!(
    project["data"]["clone_http_url"]
      .as_str()
      .unwrap()
      .ends_with("/custom-space/custom-project.git")
  );
  let id = project["data"]["id"].as_str().unwrap();
  assert_eq!(
    call(
      &router,
      "GET",
      &format!("/api/v1/repos/{id}"),
      "alice",
      json!({})
    )
    .await["data"]["key"],
    "custom-project"
  );
}
