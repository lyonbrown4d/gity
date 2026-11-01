pub(super) use super::*;
pub(super) use crate::features::test_support::fixture;
pub(super) use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
pub(super) use base64::{Engine, engine::general_purpose::STANDARD};
pub(super) use sha2::{Digest, Sha256};
pub(super) use tower::ServiceExt;
pub(super) async fn req(
  s: &AppState,
  method: &str,
  path: &str,
  user: &str,
  body: Body,
  headers: &[(&str, &str)],
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
  let mut req = Request::builder().method(method).uri(path);
  if !user.is_empty() {
    req = req.header("authorization", format!("Bearer {user}"));
  }
  for (key, value) in headers {
    req = req.header(*key, *value);
  }
  let router = routes().with_state(s.clone());
  #[cfg(feature = "packages")]
  let router =
    router.merge(gity_package_registry::routes().with_state(crate::package_authority::state(s)));
  let response = router.oneshot(req.body(body).unwrap()).await.unwrap();
  let status = response.status();
  let headers = response.headers().clone();
  let bytes = to_bytes(response.into_body(), 20 * 1024 * 1024)
    .await
    .unwrap()
    .to_vec();
  (status, headers, bytes)
}
pub(super) async fn json_req(
  s: &AppState,
  method: &str,
  path: &str,
  user: &str,
  body: Value,
) -> (StatusCode, Value) {
  let (status, _, bytes) = req(
    s,
    method,
    path,
    user,
    Body::from(body.to_string()),
    &[("content-type", "application/json")],
  )
  .await;
  (
    status,
    serde_json::from_slice(&bytes).unwrap_or(Value::Null),
  )
}
