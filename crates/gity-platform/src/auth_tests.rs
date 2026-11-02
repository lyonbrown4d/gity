use crate::*;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;
pub(crate) async fn request(
  s: &AppState,
  method: &str,
  path: &str,
  token: &str,
  body: Value,
) -> (StatusCode, Value) {
  let mut builder = Request::builder()
    .method(method)
    .uri(format!("/api/v1{path}"))
    .header("content-type", "application/json");
  if !token.is_empty() {
    builder = builder.header("authorization", format!("Bearer {token}"));
  }
  let response = router(s.clone())
    .oneshot(builder.body(Body::from(body.to_string())).unwrap())
    .await
    .unwrap();
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
  (
    status,
    serde_json::from_slice(&bytes).unwrap_or(Value::Null),
  )
}
#[tokio::test]
async fn password_sessions_rotate_and_stored_hashes_are_not_bearer_tokens() {
  let (_dir, s) = features::test_support::fixture().await;
  let (status, token) = request(
    &s,
    "POST",
    "/users/1/tokens",
    "alice",
    json!({"name":"integration"}),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{token}");
  s.db
    .insert(
      "user_access_tokens",
      json!({"user_id":1,"name":"plaintext-not-supported","token":"old-plaintext-token"})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "GET", "/users/me", "old-plaintext-token", Value::Null)
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
  let raw = token["data"]["token"].as_str().unwrap();
  let stored = format!("sha256:{}", auth::token_hash(raw));
  assert_eq!(
    request(&s, "GET", "/users/me", raw, Value::Null).await.0,
    StatusCode::OK
  );
  assert_eq!(
    request(&s, "GET", "/users/me", &stored, Value::Null)
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
  let login = json!({"username":"newuser","password":"test-password"});
  assert_eq!(
    request(&s, "POST", "/auth/login", "", login.clone())
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
  assert!(
    s.identity
      .user_model_by_username("newuser")
      .await
      .unwrap()
      .is_none()
  );
  let (status, registered) = request(&s, "POST", "/auth/register", "", login.clone()).await;
  assert_eq!(status, StatusCode::OK, "{registered}");
  let created = request(&s, "POST", "/auth/login", "", login).await.1;
  assert_eq!(created["code"], 0);
  assert_eq!(
    s.identity
      .user_model_by_username("newuser")
      .await
      .unwrap()
      .unwrap()
      .is_super_admin,
    0
  );
  assert!(!s.identity.bootstrap_claimed().await.unwrap());
  let refresh = created["data"]["refresh_token"].as_str().unwrap();
  let body = json!({"refresh_token":refresh});
  let (status, rotated) = request(&s, "POST", "/auth/refresh", "", body.clone()).await;
  assert_eq!(status, StatusCode::OK, "{rotated}");
  assert_eq!(
    request(&s, "POST", "/auth/refresh", "", body).await.0,
    StatusCode::UNAUTHORIZED
  );
}
#[tokio::test]
async fn organization_isolation_and_owner_membership_are_enforced() {
  let (_dir, s) = features::test_support::fixture().await;
  assert_eq!(
    request(&s, "GET", "/orgs/10", "outsider", Value::Null)
      .await
      .0,
    StatusCode::FORBIDDEN
  );
  s.db
    .insert(
      "organization_members",
      json!({"organization_id":10,"user_id":1,"role":"owner"})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
  let actor = gity_core::Principal {
    user_id: 1,
    username: "alice".into(),
    is_super_admin: false,
  };
  let project = s
    .db
    .one("SELECT * FROM projects WHERE id=20", vec![])
    .await
    .unwrap()
    .unwrap();
  assert_eq!(access::level(&s, &actor, &project).await.unwrap(), 50);
  let member = s
    .db
    .one(
      "SELECT id FROM project_members WHERE project_id=20 AND user_id=1",
      vec![],
    )
    .await
    .unwrap()
    .unwrap();
  s.db
    .update(
      "project_members",
      member["id"].as_i64().unwrap(),
      json!({"role":"owner"}).as_object().unwrap().clone(),
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "DELETE", "/projects/20/members/1", "bob", Value::Null)
      .await
      .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(
      &s,
      "PATCH",
      "/projects/20/members/1",
      "bob",
      json!({"role":"guest"})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  s.db
    .update(
      "users",
      1,
      json!({"is_super_admin":1}).as_object().unwrap().clone(),
    )
    .await
    .unwrap();
  assert_eq!(
    request(
      &s,
      "POST",
      "/users",
      "alice",
      json!({"username":"invalid-password-user","password":"short"})
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );
  assert!(
    s.db
      .one(
        "SELECT id FROM users WHERE username='invalid-password-user'",
        vec![]
      )
      .await
      .unwrap()
      .is_none()
  );
}
#[test]
fn branch_rule_exact_match_then_sorted_pattern_preserves_legacy_precedence() {
  let rules = vec![
    json!({"branch_name":"release/*","rule_type":"pattern"}),
    json!({"branch_name":"release/202?","rule_type":"pattern"}),
    json!({"branch_name":"release/2026","rule_type":"exact"}),
  ];
  assert_eq!(
    repository::effective_rule(&rules, "release/2026").unwrap()["rule_type"],
    "exact"
  );
  assert_eq!(
    repository::effective_rule(&rules, "release/2025").unwrap()["branch_name"],
    "release/*"
  );
  assert!(repository::matches_rule("feature/*", "feature/a/b"));
  assert!(repository::matches_rule("release/202?", "release/2026"));
}

#[tokio::test]
async fn http_errors_share_translation_assets_and_negotiate_per_request() {
  let (_dir, state) = features::test_support::fixture().await;
  async fn localized(state: AppState, language: &str) -> Value {
    let response = router(state)
      .oneshot(
        Request::builder()
          .uri("/api/v1/users/me")
          .header("accept-language", language)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
      response.headers()["www-authenticate"],
      "Bearer realm=\"Gity\""
    );
    assert!(response.headers().contains_key("content-language"));
    serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
  }
  let (english, chinese) = tokio::join!(
    localized(state.clone(), "en"),
    localized(state, "zh-CN, en;q=0.5")
  );
  assert_eq!(english["code"], chinese["code"]);
  assert_eq!(english["message"], "authentication required");
  assert_ne!(english["message"], chinese["message"]);
}
