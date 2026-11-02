use crate::handler::McpHandler;
use crate::{McpConfig, McpConfigError};
use axum::body::{Body, to_bytes};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::any_service;
use axum::{Json, Router};
use gity_api_contracts::{ApiContext, ApiTransport, ApplicationApi};
use http::header::{ACCEPT_LANGUAGE, AUTHORIZATION, ORIGIN, WWW_AUTHENTICATE};
use http::{HeaderValue, Request, StatusCode};
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

#[derive(Clone)]
struct Gate {
  backend: Arc<dyn ApplicationApi>,
  config: McpConfig,
  permits: Arc<Semaphore>,
}

pub struct McpService {
  backend: Arc<dyn ApplicationApi>,
  config: McpConfig,
}

impl McpService {
  pub fn new(backend: Arc<dyn ApplicationApi>, config: McpConfig) -> Result<Self, McpConfigError> {
    config.validate()?;
    let mut names = std::collections::HashSet::new();
    for operation in backend.operations().iter() {
      let schema = &operation.input_schema;
      let object_input = object_input_schema(schema, schema, &mut std::collections::HashSet::new());
      if !names.insert(&operation.name) || !object_input {
        return Err(McpConfigError::InvalidOperationContract);
      }
    }
    Ok(Self { backend, config })
  }

  pub fn router(&self) -> Router {
    let handler = McpHandler::new(self.backend.clone(), self.config.clone());
    let rmcp_config = StreamableHttpServerConfig::default()
      .with_legacy_session_mode(false)
      .with_json_response(true)
      .with_allowed_hosts(self.config.allowed_hosts.clone())
      .with_allowed_origins(self.config.allowed_origins.iter().cloned())
      .enforce_origin_validation();
    let service = StreamableHttpService::new(
      move || Ok(handler.clone()),
      Arc::new(NeverSessionManager::default()),
      rmcp_config,
    );
    let gate = Gate {
      backend: self.backend.clone(),
      config: self.config.clone(),
      permits: Arc::new(Semaphore::new(self.config.max_concurrent_requests)),
    };
    Router::new()
      .route_service("/mcp", any_service(service))
      .layer(middleware::from_fn_with_state(gate, authenticate_request))
      .layer(middleware::from_fn_with_state(
        self.config.max_request_body_bytes,
        limit_request_body,
      ))
  }
}

async fn limit_request_body(
  axum::extract::State(limit): axum::extract::State<usize>,
  request: Request<Body>,
  next: Next,
) -> Response {
  if request
    .headers()
    .get(http::header::CONTENT_LENGTH)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.parse::<usize>().ok())
    .is_some_and(|length| length > limit)
  {
    return StatusCode::PAYLOAD_TOO_LARGE.into_response();
  }
  let (parts, body) = request.into_parts();
  match to_bytes(body, limit).await {
    Ok(bytes) => {
      next
        .run(Request::from_parts(parts, Body::from(bytes)))
        .await
    }
    Err(_) => StatusCode::PAYLOAD_TOO_LARGE.into_response(),
  }
}

fn object_input_schema(
  root: &serde_json::Value,
  schema: &serde_json::Value,
  visited: &mut std::collections::HashSet<String>,
) -> bool {
  if let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str) {
    if !visited.insert(reference.to_owned()) {
      return false;
    }
    let resolved = reference
      .strip_prefix('#')
      .and_then(|pointer| root.pointer(pointer));
    let mut branch = visited.clone();
    return resolved.is_some_and(|resolved| object_input_schema(root, resolved, &mut branch));
  }
  if schema.get("type").and_then(serde_json::Value::as_str) == Some("object")
    || (schema.get("type").is_none() && schema.get("properties").is_some())
  {
    return true;
  }
  if let Some(parts) = schema.get("allOf").and_then(serde_json::Value::as_array) {
    return !parts.is_empty()
      && parts
        .iter()
        .all(|part| object_input_schema(root, part, visited));
  }
  if let Some(parts) = schema
    .get("oneOf")
    .or_else(|| schema.get("anyOf"))
    .and_then(serde_json::Value::as_array)
  {
    return !parts.is_empty()
      && parts
        .iter()
        .all(|part| object_input_schema(root, part, visited));
  }
  false
}

async fn authenticate_request(
  axum::extract::State(gate): axum::extract::State<Gate>,
  request: Request<Body>,
  next: Next,
) -> Response {
  let permit = match gate.permits.clone().try_acquire_owned() {
    Ok(permit) => permit,
    Err(_) => {
      return (StatusCode::TOO_MANY_REQUESTS, Json(json!({"error":"busy"}))).into_response();
    }
  };
  let timeout = gate.config.request_timeout;
  match tokio::time::timeout(timeout, handle_authenticated(gate, permit, request, next)).await {
    Ok(response) => response,
    Err(_) => (
      StatusCode::GATEWAY_TIMEOUT,
      Json(json!({"error":"request_timeout"})),
    )
      .into_response(),
  }
}

async fn handle_authenticated(
  gate: Gate,
  _permit: OwnedSemaphorePermit,
  mut request: Request<Body>,
  next: Next,
) -> Response {
  if !origin_allowed(&gate.config, request.headers().get(ORIGIN)) {
    return (
      StatusCode::FORBIDDEN,
      Json(json!({"error":"origin_forbidden"})),
    )
      .into_response();
  }
  let authorization = request
    .headers()
    .get(AUTHORIZATION)
    .and_then(|header| header.to_str().ok())
    .map(str::to_owned);
  let language = request
    .headers()
    .get(ACCEPT_LANGUAGE)
    .and_then(|header| header.to_str().ok())
    .and_then(|value| value.split(',').next())
    .unwrap_or("en")
    .to_owned();
  let context = ApiContext::new(
    authorization,
    language,
    Uuid::now_v7().to_string(),
    ApiTransport::Mcp,
  );
  if let Err(error) = gate.backend.authenticate(&context).await {
    if error.status == StatusCode::UNAUTHORIZED.as_u16() {
      let mut response = StatusCode::UNAUTHORIZED.into_response();
      if let Ok(challenge) = HeaderValue::from_str(&format!(
        "Bearer resource_metadata=\"{}\"",
        gate.config.oauth_resource_metadata_url
      )) {
        response.headers_mut().insert(WWW_AUTHENTICATE, challenge);
      }
      return response;
    }
    let status = if error.status == StatusCode::FORBIDDEN.as_u16() {
      StatusCode::FORBIDDEN
    } else {
      StatusCode::SERVICE_UNAVAILABLE
    };
    return (status, Json(json!({"error":error.code}))).into_response();
  }
  request.extensions_mut().insert(context);
  next.run(request).await
}

fn origin_allowed(config: &McpConfig, origin: Option<&HeaderValue>) -> bool {
  match origin {
    None => true,
    Some(value) => value.to_str().ok().is_some_and(|origin| {
      let Ok(origin) = url::Url::parse(origin) else {
        return false;
      };
      let origin = origin.origin().ascii_serialization();
      config.allowed_origins.iter().any(|allowed| {
        url::Url::parse(allowed)
          .is_ok_and(|allowed| allowed.origin().ascii_serialization() == origin)
      })
    }),
  }
}
