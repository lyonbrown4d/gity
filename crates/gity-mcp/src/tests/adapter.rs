use crate::{McpConfig, McpService};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use gity_api_contracts::{
  ApiContext, ApiError, ApiResult, ApplicationApi, OperationContract, OperationKind,
};
use http::{Request, StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tower::ServiceExt;

pub(super) struct MockApi {
  revoked: AtomicBool,
  auth_calls: AtomicUsize,
  invocations: AtomicUsize,
}

#[async_trait]
impl ApplicationApi for MockApi {
  fn operations(&self) -> Arc<[OperationContract]> {
    vec![
      operation("projects.list", OperationKind::Query),
      operation("projects.delete", OperationKind::Mutation),
      operation("projects.internal", OperationKind::Query),
    ]
    .into()
  }

  async fn authenticate(&self, context: &ApiContext) -> ApiResult<()> {
    self.auth_calls.fetch_add(1, Ordering::SeqCst);
    if self.revoked.load(Ordering::SeqCst) || context.authorization() != Some("Bearer good") {
      return Err(ApiError::new(401, "invalid_token", "Invalid access token"));
    }
    Ok(())
  }

  async fn invoke(&self, context: &ApiContext, name: &str, arguments: Value) -> ApiResult<Value> {
    if context.authorization() != Some("Bearer good") {
      return Err(ApiError::new(401, "invalid_token", "Invalid access token"));
    }
    self.invocations.fetch_add(1, Ordering::SeqCst);
    if name.ends_with("delete") {
      return Err(ApiError::new(
        403,
        "policy_denied",
        "Deletion is not permitted",
      ));
    }
    if name.ends_with("internal") {
      return Err(ApiError::new(
        500,
        "internal_error",
        "database password leaked",
      ));
    }
    if name.ends_with("list") {
      return Ok(json!(["p1", "p2"]));
    }
    Ok(json!({"operation":name,"arguments":arguments,"ok":true}))
  }
}

fn operation(name: &str, kind: OperationKind) -> OperationContract {
  OperationContract {
    name: name.into(),
    description: format!("{name} operation"),
    group: "projects".into(),
    kind,
    input_schema: json!({"type":"object","properties":{"id":{"type":"string"}}}),
    output_schema: if name.ends_with("list") {
      json!({"type":"array","items":{"type":"string"}})
    } else {
      json!({"type":"object","properties":{"ok":{"type":"boolean"}}})
    },
    destructive: name.ends_with("delete"),
    idempotent: name.ends_with("list"),
  }
}

pub(super) fn setup() -> (axum::Router, Arc<MockApi>) {
  setup_with_config(McpConfig::default())
}

pub(super) fn setup_with_config(config: McpConfig) -> (axum::Router, Arc<MockApi>) {
  let backend = Arc::new(MockApi {
    revoked: AtomicBool::new(false),
    auth_calls: AtomicUsize::new(0),
    invocations: AtomicUsize::new(0),
  });
  let service = McpService::new(backend.clone(), config).unwrap();
  (service.router(), backend)
}

pub(super) fn request(body: Value, token: Option<&str>, origin: Option<&str>) -> Request<Body> {
  let mut builder = Request::builder()
    .method("POST")
    .uri("/mcp")
    .header("host", "localhost")
    .header("content-type", "application/json")
    .header("accept", "application/json, text/event-stream");
  if let Some(token) = token {
    builder = builder.header("authorization", token);
  }
  if let Some(origin) = origin {
    builder = builder.header("origin", origin);
  }
  builder.body(Body::from(body.to_string())).unwrap()
}

pub(super) async fn json_response(response: axum::response::Response) -> Value {
  serde_json::from_slice(&to_bytes(response.into_body(), 1_000_000).await.unwrap()).unwrap()
}

#[tokio::test]
async fn lists_real_contract_tools_with_annotations_and_schemas() {
  let (router, _) = setup();
  let response = router
    .clone()
    .oneshot(request(
      json!({
        "jsonrpc":"2.0","id":1,"method":"tools/list","params":{}
      }),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = json_response(response).await;
  let tools = body["result"]["tools"].as_array().unwrap();
  assert_eq!(tools.len(), 3);
  assert_eq!(tools[0]["inputSchema"]["type"], "object");
  let delete = tools
    .iter()
    .find(|tool| tool["name"] == "projects.delete")
    .unwrap();
  assert_eq!(delete["annotations"]["readOnlyHint"], false);
  assert_eq!(delete["annotations"]["destructiveHint"], true);
  let query = tools
    .iter()
    .find(|tool| tool["name"] == "projects.list")
    .unwrap();
  assert_eq!(query["annotations"]["readOnlyHint"], true);
  assert_eq!(query["annotations"]["idempotentHint"], true);
  assert_eq!(query["outputSchema"]["type"], "object");
  assert_eq!(
    query["outputSchema"]["properties"]["result"]["type"],
    "array"
  );
}

#[tokio::test]
async fn group_filter_controls_catalog_and_rejects_hidden_tools() {
  let config = McpConfig {
    tool_groups: Some(["other".to_owned()].into()),
    ..McpConfig::default()
  };
  let (router, _) = setup_with_config(config);
  let list = router
    .clone()
    .oneshot(request(
      json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  let body = json_response(list).await;
  assert_eq!(body["result"]["tools"].as_array().unwrap().len(), 0);
  let call = router
    .oneshot(request(
      json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"projects.list","arguments":{}}}),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  let body = json_response(call).await;
  assert_eq!(body["error"]["code"], -32601);
}

#[tokio::test]
async fn calls_return_structured_content_and_recheck_revocation() {
  let (router, backend) = setup();
  let response = router
    .clone()
    .oneshot(request(
      json!({
        "jsonrpc":"2.0","id":3,"method":"tools/call",
        "params":{"name":"projects.list","arguments":{"id":"p1"}}
      }),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  let body = json_response(response).await;
  assert_eq!(body["result"]["structuredContent"]["result"][0], "p1");
  assert_eq!(backend.invocations.load(Ordering::SeqCst), 1);
  backend.revoked.store(true, Ordering::SeqCst);
  let revoked = router
    .oneshot(request(
      json!({
        "jsonrpc":"2.0","id":4,"method":"tools/call",
        "params":{"name":"projects.list","arguments":{}}
      }),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn backend_errors_are_tool_errors_with_stable_codes() {
  let (router, _) = setup();
  let response = router
    .oneshot(request(
      json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"projects.delete","arguments":{"id":"p1"}}}),
      Some("Bearer good"),
      None,
    ))
    .await
    .unwrap();
  let body = json_response(response).await;
  assert_eq!(body["result"]["isError"], true);
  assert_eq!(
    body["result"]["structuredContent"]["error"]["code"],
    "policy_denied"
  );
}

#[tokio::test]
async fn invalid_credentials_and_origins_are_rejected_before_dispatch() {
  let (router, backend) = setup();
  let invalid = router
    .clone()
    .oneshot(request(
      json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
      Some("Bearer bad"),
      None,
    ))
    .await
    .unwrap();
  assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
  assert!(invalid.headers().contains_key("www-authenticate"));
  let origin = router
    .oneshot(request(
      json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
      Some("Bearer good"),
      Some("https://attacker.test"),
    ))
    .await
    .unwrap();
  assert_eq!(origin.status(), StatusCode::FORBIDDEN);
  assert_eq!(backend.invocations.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn each_http_request_authenticates_with_its_own_credentials() {
  let (router, backend) = setup();
  for token in ["Bearer good", "Bearer bad"] {
    let _ = router
      .clone()
      .oneshot(request(
        json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        Some(token),
        None,
      ))
      .await
      .unwrap();
  }
  assert_eq!(backend.auth_calls.load(Ordering::SeqCst), 2);
}
