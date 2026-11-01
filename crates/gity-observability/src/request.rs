use axum::{
  extract::{MatchedPath, Request},
  middleware::Next,
  response::Response,
};
use opentelemetry::{global, propagation::Extractor};
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
struct Headers<'a>(&'a axum::http::HeaderMap);
impl Extractor for Headers<'_> {
  fn get(&self, key: &str) -> Option<&str> {
    self.0.get(key).and_then(|v| v.to_str().ok())
  }
  fn keys(&self) -> Vec<&str> {
    self.0.keys().map(|v| v.as_str()).collect()
  }
}
pub async fn instrument(mut request: Request, next: Next) -> Response {
  let route = request
    .extensions()
    .get::<MatchedPath>()
    .map(|v| v.as_str())
    .unwrap_or("unmatched")
    .to_owned();
  let method = match request.method().as_str() {
    "GET" => "GET",
    "POST" => "POST",
    "PUT" => "PUT",
    "PATCH" => "PATCH",
    "DELETE" => "DELETE",
    "HEAD" => "HEAD",
    "OPTIONS" => "OPTIONS",
    _ => "OTHER",
  };
  let id = request
    .headers()
    .get("x-request-id")
    .and_then(|v| v.to_str().ok())
    .filter(|v| {
      v.len() <= 64 && !v.is_empty() && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    })
    .map(str::to_owned)
    .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
  if let Ok(value) = id.parse() {
    request.headers_mut().insert("x-request-id", value);
  }
  let context = global::get_text_map_propagator(|p| p.extract(&Headers(request.headers())));
  let span = tracing::info_span!("http.request",request_id=%id,http.route=%route,http.request.method=method,otel.kind="server",http.response.status_code=tracing::field::Empty);
  let _ = span.set_parent(context);
  let started = std::time::Instant::now();
  let mut response = next.run(request).instrument(span.clone()).await;
  let status = response.status().as_u16();
  span.record("http.response.status_code", status);
  metrics::counter!("gity_http_requests_total","route"=>route.clone(),"method"=>method,"status"=>status.to_string()).increment(1);
  metrics::histogram!("gity_http_request_duration_seconds","route"=>route,"method"=>method)
    .record(started.elapsed().as_secs_f64());
  if let Ok(id) = id.parse() {
    response.headers_mut().insert("x-request-id", id);
  }
  response
}
