use super::*;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use gity_identity::{AuthorizationRequest, ClientRegistration, TokenRequest};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tower::ServiceExt;

async fn oauth(state: &AppState, user: i64, scopes: &str) -> String {
  let client = state
    .oauth
    .register_client(ClientRegistration {
      client_name: "protocol-test".into(),
      redirect_uris: vec!["http://127.0.0.1:42000/callback".into()],
    })
    .await
    .unwrap();
  let id = client["client_id"].as_str().unwrap().to_owned();
  let verifier = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";
  let resource = format!("{}/mcp", state.settings.base_url.trim_end_matches('/'));
  let redirect = state
    .oauth
    .begin_authorization(AuthorizationRequest {
      client_id: id.clone(),
      redirect_uri: "http://127.0.0.1:42000/callback".into(),
      response_type: "code".into(),
      code_challenge: URL_SAFE_NO_PAD.encode(Sha256::digest(verifier)),
      code_challenge_method: "S256".into(),
      scope: scopes.into(),
      state: "protocol-test-state".into(),
      resource: resource.clone(),
      nonce: None,
    })
    .await
    .unwrap();
  let location = reqwest::Url::parse(&redirect).unwrap();
  let pending = location
    .query_pairs()
    .find(|(key, _)| key == "request_id")
    .unwrap()
    .1
    .into_owned();
  state.oauth.pending_details(&pending, user).await.unwrap();
  let callback = state
    .oauth
    .decide(&pending, user, true, false)
    .await
    .unwrap();
  let callback = reqwest::Url::parse(&callback).unwrap();
  let code = callback
    .query_pairs()
    .find(|(key, _)| key == "code")
    .unwrap()
    .1
    .into_owned();
  state
    .oauth
    .token(TokenRequest {
      grant_type: "authorization_code".into(),
      code: Some(code),
      refresh_token: None,
      client_id: id,
      redirect_uri: Some("http://127.0.0.1:42000/callback".into()),
      code_verifier: Some(verifier.into()),
      resource: Some(resource),
    })
    .await
    .unwrap()
    .access_token
}
async fn post(router: &axum::Router, path: &str, token: &str, body: Value) -> (StatusCode, Value) {
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri(path)
        .header("host", "localhost")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap(),
    )
    .await
    .unwrap();
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
    .await
    .unwrap();
  let value = serde_json::from_slice(&bytes)
    .unwrap_or_else(|_| json!({"raw":String::from_utf8_lossy(&bytes)}));
  (status, value)
}
#[tokio::test]
async fn graphql_and_mcp_use_real_project_workflow_and_revalidate_revocation() {
  let (_directory, state) = crate::features::test_support::fixture().await;
  let catalog = catalog::build().unwrap();
  let detail = catalog
    .iter()
    .find(|entry| entry.registration.handler.ends_with("::projects::detail"))
    .unwrap()
    .contract
    .name
    .clone();
  let module = fluxdi::Injector::root();
  module::configure(&state, &module).unwrap();
  let router = crate::router_with_apis(
    state.clone(),
    module.try_resolve::<module::ApiRoutes>().unwrap().0.clone(),
  );
  let query = format!("{{{detail}(path:{{id:\"20\"}}){{id name}}}}");
  let (_, body) = post(&router, "/api/graphql", "alice", json!({"query":query})).await;
  assert!(body.get("errors").is_none(), "{body}");
  assert_eq!(body["data"][&detail]["id"], "20");
  let token = oauth(&state, 1, "mcp read_api").await;
  let (status, body) = post(
    &router,
    "/mcp",
    &token,
    json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(
    body["result"]["tools"].as_array().unwrap().len(),
    catalog.len()
  );
  let call = json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":detail,"arguments":{"path":{"id":"20"}}}});
  let (status, body) = post(&router, "/mcp", &token, call.clone()).await;
  assert_eq!(status, StatusCode::OK, "{body}");
  assert_eq!(
    body["result"]["structuredContent"]["result"]["id"], "20",
    "{body}"
  );
  let other = oauth(&state, 3, "mcp read_api").await;
  let (_, body) = post(&router, "/mcp", &other, call.clone()).await;
  assert_eq!(body["result"]["isError"], true, "{body}");
  let (status, _) = post(
    &router,
    "/api/graphql",
    &token,
    json!({"query":"{__typename}"}),
  )
  .await;
  assert_eq!(status, StatusCode::UNAUTHORIZED);
  state.oauth.revoke(&token).await.unwrap();
  let (status, _) = post(&router, "/mcp", &token, call).await;
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}
