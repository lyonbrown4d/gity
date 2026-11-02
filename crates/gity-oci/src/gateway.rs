use crate::{AuthorizationError, MirrorResolver, OciAuthorization, OciRequest, OciSettings};
use crate::{
  gateway_headers::{
    is_hop_header, is_identity_header, map_storage_location, validate_auth_realm, validate_redirect,
  },
  gateway_paths::{
    is_sha256_digest, map_mirror_error, parse_request, replace_manifest_reference, target_uri,
    validate_storage_path,
  },
  mirror::resolver_is_configured,
};
use axum::{
  Router,
  body::Body,
  extract::State,
  http::{HeaderName, HeaderValue, Method, Request, StatusCode},
  response::Response,
  routing::{any, get},
};
use futures_util::StreamExt;
use std::sync::Arc;

#[path = "gateway_health.rs"]
mod health;

#[derive(Clone)]
pub struct OciGateway {
  settings: Arc<OciSettings>,
  authorizer: Arc<dyn OciAuthorization>,
  mirror_resolver: Option<Arc<dyn MirrorResolver>>,
  client: reqwest::Client,
}

impl OciGateway {
  pub fn new(settings: OciSettings, authorizer: Arc<dyn OciAuthorization>) -> Result<Self, String> {
    Self::new_with_mirror(settings, authorizer, None)
  }

  pub fn new_with_mirror(
    settings: OciSettings,
    authorizer: Arc<dyn OciAuthorization>,
    mirror_resolver: Option<Arc<dyn MirrorResolver>>,
  ) -> Result<Self, String> {
    settings.validate()?;
    let client = reqwest::Client::builder()
      .timeout(settings.request_timeout)
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .map_err(|error| error.to_string())?;
    Ok(Self {
      settings: Arc::new(settings),
      authorizer,
      mirror_resolver,
      client,
    })
  }
}

pub fn router(gateway: OciGateway) -> Router {
  Router::new()
    .route("/readyz", get(health::ready))
    .route("/v2", any(handle))
    .route("/v2/", any(handle))
    .route("/v2/{*path}", any(handle))
    .with_state(gateway)
    .layer(axum::middleware::from_fn(gity_observability::instrument))
}

async fn handle(State(gateway): State<OciGateway>, request: Request<Body>) -> Response {
  match forward(gateway, request).await {
    Ok(response) => response,
    Err((status, message)) => {
      let mut response = (status, message).into_response();
      if status == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
          http::header::WWW_AUTHENTICATE,
          HeaderValue::from_static("Basic realm=\"Gity registry\""),
        );
        response.headers_mut().insert(
          "docker-distribution-api-version",
          HeaderValue::from_static("registry/2.0"),
        );
      }
      response
    }
  }
}

use axum::response::IntoResponse;

async fn forward(
  gateway: OciGateway,
  request: Request<Body>,
) -> Result<Response, (StatusCode, &'static str)> {
  let method = request.method().clone();
  let uri = request.uri().clone();
  let headers = request.headers().clone();
  let (repo, operation, source, reference) = parse_request(&method, &uri)?;
  let authorization = headers
    .get(http::header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok());
  let decision = gateway
    .authorizer
    .authorize(
      &OciRequest {
        repository: repo.clone(),
        method: method.to_string(),
        operation,
        mount_source: source,
        reference: reference.clone(),
      },
      authorization,
    )
    .await
    .map_err(|error| match error {
      AuthorizationError::Unauthenticated => (StatusCode::UNAUTHORIZED, "authentication required"),
      AuthorizationError::Forbidden => (StatusCode::FORBIDDEN, "operation forbidden"),
      AuthorizationError::NotFound => (StatusCode::NOT_FOUND, "repository not found"),
      AuthorizationError::Unavailable => (
        StatusCode::SERVICE_UNAVAILABLE,
        "authorization service unavailable",
      ),
    })?;
  // Configured mirror sources enable caching alongside hosted repositories.
  // Only a repository selected as a mirror by the ACL adapter is read-only.
  if decision.mirror_source.is_some()
    && matches!(
      operation,
      crate::OciOperation::Push | crate::OciOperation::Delete
    )
  {
    return Err((
      StatusCode::METHOD_NOT_ALLOWED,
      "mirror registry is read-only",
    ));
  }
  if !repo.is_empty() {
    validate_storage_path(&decision.storage_repository)?;
  }
  if let Some(source) = &decision.mount_source_storage_repository {
    validate_storage_path(source)?;
  }
  let mut effective_uri = uri.clone();
  if let Some(mirror_source) = &decision.mirror_source {
    if !resolver_is_configured(&gateway.settings, mirror_source)
      || mirror_source.reference.as_deref() != reference.as_deref()
    {
      return Err((StatusCode::BAD_GATEWAY, "invalid mirror source mapping"));
    }
    if matches!(method, Method::GET | Method::HEAD) && reference.is_some() {
      let resolver = gateway.mirror_resolver.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "mirror resolver unavailable",
      ))?;
      let digest = resolver
        .resolve_digest(mirror_source)
        .await
        .map_err(map_mirror_error)?;
      if !is_sha256_digest(&digest) {
        return Err((StatusCode::BAD_GATEWAY, "invalid upstream manifest digest"));
      }
      effective_uri = replace_manifest_reference(&uri, &digest)?;
    }
  }
  let target = target_uri(&gateway.settings.upstream, &effective_uri, &repo, &decision)?;
  let mut outgoing = gateway
    .client
    .request(method, target)
    .header(http::header::ACCEPT_ENCODING, "identity")
    .header("x-angos-no-redirect", "true");
  for (name, value) in headers.iter() {
    if is_hop_header(name, &headers)
      || is_identity_header(name)
      || name == http::header::AUTHORIZATION
      || name == http::header::HOST
      || name == http::header::CONTENT_LENGTH
    {
      continue;
    }
    outgoing = outgoing.header(name, value);
  }
  let body = request
    .into_body()
    .into_data_stream()
    .map(|part| part.map_err(std::io::Error::other));
  let result = outgoing
    .body(reqwest::Body::wrap_stream(body))
    .send()
    .await
    .map_err(|_| (StatusCode::BAD_GATEWAY, "registry upstream unavailable"))?;
  let status = StatusCode::from_u16(result.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
  let upstream_headers = result.headers().clone();
  if status == StatusCode::UNAUTHORIZED
    && let Some(challenge) = upstream_headers
      .get(http::header::WWW_AUTHENTICATE)
      .and_then(|value| value.to_str().ok())
  {
    validate_auth_realm(challenge, &gateway.settings.upstream_policy.allowed_hosts)?;
  }
  let mut upstream_body = result.bytes_stream();
  let body = if status.is_success() && uri.path().ends_with("/tags/list") {
    let mut bytes = Vec::new();
    while let Some(chunk) = upstream_body.next().await {
      let chunk =
        chunk.map_err(|_| (StatusCode::BAD_GATEWAY, "registry metadata stream failed"))?;
      if bytes.len().saturating_add(chunk.len()) > 1024 * 1024 {
        return Err((
          StatusCode::BAD_GATEWAY,
          "registry metadata exceeds size limit",
        ));
      }
      bytes.extend_from_slice(&chunk);
    }
    let mut metadata: serde_json::Value = serde_json::from_slice(&bytes)
      .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry metadata"))?;
    metadata["name"] = serde_json::Value::String(repo.clone());
    let bytes = serde_json::to_vec(&metadata)
      .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry metadata"))?;
    Some(bytes)
  } else {
    None
  };
  let mut response = if let Some(bytes) = body.as_ref() {
    Response::new(Body::from(bytes.clone()))
  } else {
    let stream = upstream_body.map(|part| part.map_err(std::io::Error::other));
    Response::new(Body::from_stream(stream))
  };
  *response.status_mut() = status;
  for (name, value) in upstream_headers.iter() {
    if is_hop_header(name, &upstream_headers) {
      continue;
    }
    if name == http::header::LOCATION {
      let location = value
        .to_str()
        .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry redirect"))?;
      let rewritten = validate_redirect(location, &gateway.settings.upstream)?;
      if let Some(location) = rewritten {
        let location = map_storage_location(&location, &decision.storage_repository, &repo)?;
        response.headers_mut().insert(
          http::header::LOCATION,
          HeaderValue::from_str(&location)
            .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry redirect"))?,
        );
        continue;
      }
    }
    if body.is_some() && name == http::header::CONTENT_LENGTH {
      continue;
    }
    if body.is_some() && name == http::header::CONTENT_ENCODING {
      continue;
    }
    if let (Ok(name), Ok(value)) = (
      HeaderName::from_bytes(name.as_str().as_bytes()),
      HeaderValue::from_bytes(value.as_bytes()),
    ) {
      response.headers_mut().append(name, value);
    }
  }
  if let Some(bytes) = body {
    response.headers_mut().insert(
      http::header::CONTENT_LENGTH,
      HeaderValue::from_str(&bytes.len().to_string())
        .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry metadata"))?,
    );
  }
  Ok(response)
}

#[cfg(test)]
#[path = "mirror_tests.rs"]
mod mirror_tests;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "gateway_probe_tests.rs"]
mod probe_tests;

#[cfg(test)]
#[path = "docker_handshake_tests.rs"]
mod docker_handshake_tests;
