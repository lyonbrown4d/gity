use super::*;
use axum::http::StatusCode;
use axum::{
  body::{Body, to_bytes},
  http::Request,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tower::ServiceExt;

#[tokio::test]
async fn oauth_http_metadata_registration_consent_and_form_token_exchange() {
  let (_dir, state) = crate::features::test_support::fixture().await;
  let owner = state
    .identity
    .user_by_username("alice")
    .await
    .unwrap()
    .unwrap();
  let session_token = "oauth-browser-session-token";
  let now = chrono::Utc::now();
  state
    .identity
    .create_session_pair(
      owner.id,
      crate::auth::token_hash(session_token),
      crate::auth::token_hash("oauth-browser-refresh-token"),
      "oauth-browser-session-family".into(),
      now + chrono::Duration::hours(1),
      now + chrono::Duration::days(1),
    )
    .await
    .unwrap();
  let app = routes().with_state(state.clone());
  let metadata = app
    .clone()
    .oneshot(
      Request::get("/.well-known/oauth-authorization-server")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(metadata.status(), StatusCode::OK);
  let metadata_body = to_bytes(metadata.into_body(), 4096).await.unwrap();
  let metadata: Value = serde_json::from_slice(&metadata_body).unwrap();
  assert_eq!(
    metadata["token_endpoint"],
    "http://localhost:8080/oauth/token"
  );
  assert_eq!(metadata["code_challenge_methods_supported"][0], "S256");
  let resource_response = app
    .clone()
    .oneshot(
      Request::get("/.well-known/oauth-protected-resource/mcp")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let resource_body = to_bytes(resource_response.into_body(), 4096).await.unwrap();
  let resource: Value = serde_json::from_slice(&resource_body).unwrap();
  assert_eq!(resource["resource"], "http://localhost:8080/mcp");

  let registration = app
    .clone()
    .oneshot(
      Request::post("/oauth/register")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"client_name":"Browser MCP","redirect_uris":["http://127.0.0.1:49991/callback"]}"#,
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(registration.status(), StatusCode::CREATED);
  let registration_body = to_bytes(registration.into_body(), 4096).await.unwrap();
  let client: Value = serde_json::from_slice(&registration_body).unwrap();
  let client_id = client["client_id"].as_str().unwrap();

  let verifier = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";
  let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
  let query = url::form_urlencoded::Serializer::new(String::new())
    .append_pair("client_id", client_id)
    .append_pair("redirect_uri", "http://127.0.0.1:49991/callback")
    .append_pair("response_type", "code")
    .append_pair("code_challenge", &challenge)
    .append_pair("code_challenge_method", "S256")
    .append_pair("scope", "mcp read_api")
    .append_pair("state", "state-value-with-enough-entropy")
    .append_pair("resource", "http://localhost:8080/mcp")
    .finish();
  let authorize = app
    .clone()
    .oneshot(
      Request::get(format!("/oauth/authorize?{query}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(authorize.status(), StatusCode::TEMPORARY_REDIRECT);
  let consent_url = authorize
    .headers()
    .get("location")
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned();
  let request_id = url::Url::parse(&consent_url)
    .unwrap()
    .query_pairs()
    .find(|(name, _)| name == "request_id")
    .unwrap()
    .1
    .into_owned();

  let consent_path = format!("/api/v1/oauth/requests/{request_id}");
  let pending = app
    .clone()
    .oneshot(
      Request::get(&consent_path)
        .header("authorization", format!("Bearer {session_token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let pending_status = pending.status();
  let pending_body = to_bytes(pending.into_body(), 4096).await.unwrap();
  assert_eq!(
    pending_status,
    StatusCode::OK,
    "{}",
    String::from_utf8_lossy(&pending_body)
  );
  let pending: Value = serde_json::from_slice(&pending_body).unwrap();
  assert_eq!(pending["data"]["client_name"], "Browser MCP");

  let decision = app
    .clone()
    .oneshot(
      Request::post(&consent_path)
        .header("authorization", format!("Bearer {session_token}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"approve":true}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(decision.status(), StatusCode::OK);
  let decision_body = to_bytes(decision.into_body(), 4096).await.unwrap();
  let decision: Value = serde_json::from_slice(&decision_body).unwrap();
  let callback = url::Url::parse(decision["data"]["redirect_uri"].as_str().unwrap()).unwrap();
  let code = callback
    .query_pairs()
    .find(|(name, _)| name == "code")
    .unwrap()
    .1
    .into_owned();

  let form = url::form_urlencoded::Serializer::new(String::new())
    .append_pair("grant_type", "authorization_code")
    .append_pair("code", &code)
    .append_pair("client_id", client_id)
    .append_pair("redirect_uri", "http://127.0.0.1:49991/callback")
    .append_pair("code_verifier", verifier)
    .append_pair("resource", "http://localhost:8080/mcp")
    .finish();
  let token = app
    .clone()
    .oneshot(
      Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form.clone()))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(token.status(), StatusCode::OK);
  assert_eq!(token.headers().get("cache-control").unwrap(), "no-store");
  assert_eq!(token.headers().get("pragma").unwrap(), "no-cache");
  let token_body = to_bytes(token.into_body(), 4096).await.unwrap();
  let token: Value = serde_json::from_slice(&token_body).unwrap();
  let access_token = token["access_token"].as_str().unwrap();
  assert!(access_token.starts_with("gity_oauth_"));
  assert_eq!(
    state
      .oauth
      .validate_bearer(access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .unwrap()
      .user_id,
    owner.id
  );
  let revoke_form = url::form_urlencoded::Serializer::new(String::new())
    .append_pair("token", access_token)
    .finish();
  let revoked = app
    .clone()
    .oneshot(
      Request::post("/oauth/revoke")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(revoke_form))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(revoked.status(), StatusCode::OK);
  assert!(
    state
      .oauth
      .validate_bearer(access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .is_none()
  );
}
