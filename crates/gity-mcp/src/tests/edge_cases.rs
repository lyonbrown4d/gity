use super::adapter::{json_response, request, setup, setup_with_config};
use crate::McpConfig;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn internal_error_details_are_redacted_and_structured_results_are_objects() {
  let (router, _) = setup();
  let response = router
    .oneshot(request(
      json!({"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"projects.internal","arguments":{}}}),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  let body = json_response(response).await;
  let result = &body["result"];
  assert_eq!(result["isError"], true);
  assert!(result["structuredContent"].is_object());
  assert_eq!(
    result["structuredContent"]["error"]["message"],
    "Operation failed"
  );
  assert!(!result.to_string().contains("database password"));
}

#[tokio::test]
async fn request_body_limit_is_enforced() {
  let config = McpConfig {
    max_request_body_bytes: 16,
    ..McpConfig::default()
  };
  let (router, _) = setup_with_config(config);
  let response = router
    .oneshot(request(
      json!({"oversized":"request body"}),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn disallowed_host_is_rejected_by_streamable_http_transport() {
  let (router, _) = setup();
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", "attacker.test")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("authorization", "Bearer good")
        .body(Body::from(
          json!({"jsonrpc":"2.0","id":9,"method":"tools/list"}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  assert!(!response.status().is_success());
}
