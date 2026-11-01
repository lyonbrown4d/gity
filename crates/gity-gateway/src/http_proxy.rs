use crate::GatewayHealth;
use axum::{
  Router,
  body::Body,
  extract::State,
  http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
  response::{IntoResponse, Response},
  routing::any,
};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(super) struct HttpProxy {
  origin: Arc<str>,
  package_origin: Option<Arc<str>>,
  public_scheme: Arc<str>,
  client: reqwest::Client,
  request_slots: Arc<Semaphore>,
  health: GatewayHealth,
}

impl HttpProxy {
  pub(super) fn with_public_scheme(mut self, scheme: String) -> Self {
    self.public_scheme = Arc::from(scheme);
    self
  }

  pub(super) fn with_package_origin(mut self, origin: Option<String>) -> Self {
    self.package_origin = origin.map(|origin| Arc::from(origin.trim_end_matches('/')));
    self
  }

  pub(super) fn new(
    origin: String,
    request_timeout: std::time::Duration,
    max_requests: usize,
    health: GatewayHealth,
  ) -> Result<Self, String> {
    let client = reqwest::Client::builder()
      .timeout(request_timeout)
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .map_err(|error| error.to_string())?;
    Ok(Self {
      origin: Arc::from(origin.trim_end_matches('/')),
      package_origin: None,
      public_scheme: Arc::from("http"),
      client,
      request_slots: Arc::new(Semaphore::new(max_requests)),
      health,
    })
  }
}

pub(super) fn router(proxy: HttpProxy) -> Router {
  Router::new()
    .route("/", any(forward))
    .route("/{*path}", any(forward))
    .with_state(proxy)
    .layer(axum::middleware::from_fn(gity_observability::instrument))
}

async fn forward(State(proxy): State<HttpProxy>, request: Request<Body>) -> Response {
  if request.uri().path() == "/healthz" || request.uri().path() == "/readyz" {
    return if proxy.health.is_healthy() {
      StatusCode::OK.into_response()
    } else {
      StatusCode::SERVICE_UNAVAILABLE.into_response()
    };
  }
  let permit = match proxy.request_slots.clone().try_acquire_owned() {
    Ok(permit) => permit,
    Err(_) => {
      return (
        StatusCode::SERVICE_UNAVAILABLE,
        "gateway request limit reached",
      )
        .into_response();
    }
  };
  match forward_request(proxy, request, permit).await {
    Ok(response) => response,
    Err(status) => (status, "main service unavailable").into_response(),
  }
}

async fn forward_request(
  proxy: HttpProxy,
  request: Request<Body>,
  permit: tokio::sync::OwnedSemaphorePermit,
) -> Result<Response, StatusCode> {
  let (parts, body) = request.into_parts();
  let origin = if package_path(parts.uri.path()) {
    proxy.package_origin.as_deref().unwrap_or(&proxy.origin)
  } else {
    &proxy.origin
  };
  let target = target_uri(origin, parts.uri.path(), parts.uri.query());
  let mut outgoing = proxy
    .client
    .request(parts.method, target)
    .header(http::header::ACCEPT_ENCODING, "identity");
  for (name, value) in &parts.headers {
    if should_drop_header(name, &parts.headers) {
      continue;
    }
    outgoing = outgoing.header(name, value);
  }
  if let Some(host) = parts.headers.get(http::header::HOST) {
    outgoing = outgoing
      .header(http::header::HOST, host)
      .header("x-forwarded-host", host)
      .header("x-forwarded-proto", proxy.public_scheme.as_ref());
  }
  let body = body
    .into_data_stream()
    .map(|chunk| chunk.map_err(std::io::Error::other));
  let upstream = outgoing
    .body(reqwest::Body::wrap_stream(body))
    .send()
    .await
    .map_err(|_| StatusCode::BAD_GATEWAY)?;
  let status =
    StatusCode::from_u16(upstream.status().as_u16()).map_err(|_| StatusCode::BAD_GATEWAY)?;
  let upstream_headers = upstream.headers().clone();
  let mut response = Response::new(Body::from_stream(upstream.bytes_stream().map(
    move |chunk| {
      let _keep_slot_until_body_is_consumed = &permit;
      chunk.map_err(std::io::Error::other)
    },
  )));
  *response.status_mut() = status;
  for (name, value) in &upstream_headers {
    if should_drop_header(name, &upstream_headers) {
      continue;
    }
    if let (Ok(name), Ok(value)) = (
      HeaderName::from_bytes(name.as_str().as_bytes()),
      HeaderValue::from_bytes(value.as_bytes()),
    ) {
      response.headers_mut().append(name, value);
    }
  }
  Ok(response)
}

fn target_uri(origin: &str, path: &str, query: Option<&str>) -> String {
  let path = if path.starts_with('/') { path } else { "/" };
  match query {
    Some(query) => format!("{origin}{path}?{query}"),
    None => format!("{origin}{path}"),
  }
}

fn should_drop_header(name: &HeaderName, headers: &HeaderMap) -> bool {
  let name_text = name.as_str();
  if matches!(
    name_text,
    "connection"
      | "keep-alive"
      | "proxy-authenticate"
      | "proxy-authorization"
      | "te"
      | "trailer"
      | "transfer-encoding"
      | "upgrade"
      | "host"
      | "content-length"
      | "forwarded"
  ) || name_text.starts_with("x-forwarded-")
    || name_text.starts_with("x-gity-")
    || matches!(
      name_text,
      "x-user-id" | "x-authenticated-user" | "x-remote-user"
    )
  {
    return true;
  }
  headers
    .get_all(http::header::CONNECTION)
    .iter()
    .filter_map(|value| value.to_str().ok())
    .flat_map(|value| value.split(','))
    .any(|token| token.trim().eq_ignore_ascii_case(name_text))
}

#[cfg(test)]
#[path = "http_proxy_tests.rs"]
mod tests;

fn package_path(path: &str) -> bool {
  let Some(project_path) = path
    .strip_prefix("/api/v1/projects/")
    .or_else(|| path.strip_prefix("/api/v1/repos/"))
  else {
    return false;
  };
  let Some((project, rest)) = project_path.split_once('/') else {
    return false;
  };
  !project.is_empty()
    && project.bytes().all(|byte| byte.is_ascii_digit())
    && (rest == "packages" || rest.starts_with("packages/"))
}
