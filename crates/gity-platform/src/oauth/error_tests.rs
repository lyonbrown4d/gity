use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
#[tokio::test]
async fn token_form_failures_have_oauth_errors_and_never_cache_credentials() {
  let (_dir, state) = crate::features::test_support::fixture().await;
  let router = super::routes().with_state(state);
  for (form, expected) in [
    ("grant_type=authorization_code", "invalid_request"),
    (
      "client_id=client&grant_type=password",
      "unsupported_grant_type",
    ),
    (
      "client_id=client&grant_type=authorization_code&code=bad&redirect_uri=http%3A%2F%2Flocalhost%2Fcallback&code_verifier=abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
      "invalid_grant",
    ),
  ] {
    let response = router
      .clone()
      .oneshot(
        Request::post("/oauth/token")
          .header("content-type", "application/x-www-form-urlencoded")
          .body(Body::from(form))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(response.headers()["pragma"], "no-cache");
    let body: Value =
      serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["error"], expected);
  }
}
