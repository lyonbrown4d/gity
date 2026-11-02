use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::AppState;

#[utoipa::path(
  get,
  path = "/.well-known/oauth-authorization-server",
  tag = "oauth",
  description = "OAuth authorization server metadata for public PKCE clients.",
  responses((status = 200, description = "OAuth authorization server metadata", body = Value))
)]
pub(super) async fn authorization_metadata(State(state): State<AppState>) -> Json<Value> {
  Json(state.oauth.metadata())
}

#[utoipa::path(
  get,
  path = "/.well-known/oauth-protected-resource",
  tag = "oauth",
  description = "Protected resource metadata for the MCP resource server.",
  responses((status = 200, description = "Protected resource metadata", body = Value))
)]
pub(super) async fn resource_metadata(State(state): State<AppState>) -> Json<Value> {
  Json(resource_value(&state))
}

#[utoipa::path(
  get,
  path = "/.well-known/oauth-protected-resource/mcp",
  tag = "oauth",
  description = "Protected resource metadata for the MCP resource server.",
  responses((status = 200, description = "Protected resource metadata", body = Value))
)]
pub(super) async fn mcp_resource_metadata(State(state): State<AppState>) -> Json<Value> {
  Json(resource_value(&state))
}

fn resource_value(state: &AppState) -> Value {
  let resource = format!("{}/mcp", state.settings.base_url.trim_end_matches('/'));
  json!({
    "resource": resource,
    "authorization_servers": [state.settings.base_url.trim_end_matches('/')],
    "bearer_methods_supported": ["header"],
    "scopes_supported": ["mcp", "read_api", "write_api", "admin_mode"]
  })
}
