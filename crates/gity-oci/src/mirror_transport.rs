use futures_util::{StreamExt, TryStreamExt};
use http::{HeaderMap, Method, Request, Response, StatusCode, Uri, header::LOCATION};
use http_body_util::BodyExt;
use oci_client::transport::{self, Body};
use std::{collections::BTreeSet, io, sync::Arc, time::Duration};
use tower::service_fn;

const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_REDIRECTS: usize = 4;

#[cfg(test)]
#[path = "mirror_transport_tests.rs"]
mod tests;

pub(crate) fn build_transport(
  allowed_hosts: BTreeSet<String>,
  timeout: Duration,
) -> Result<oci_client::Client, String> {
  let client = reqwest::Client::builder()
    .timeout(timeout)
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .map_err(|_| "cannot initialize mirror HTTP transport".to_owned())?;
  let allowed_hosts = Arc::new(allowed_hosts);
  let service = service_fn(move |request: transport::Request| {
    let client = client.clone();
    let allowed_hosts = allowed_hosts.clone();
    async move { send_checked(client, allowed_hosts, request).await }
  });
  Ok(oci_client::Client::new_with_transport(
    oci_client::client::ClientConfig::default(),
    service,
  ))
}

async fn send_checked(
  client: reqwest::Client,
  allowed_hosts: Arc<BTreeSet<String>>,
  mut request: Request<Body>,
) -> Result<Response<Body>, transport::BoxError> {
  let original_origin = request.uri().clone();
  let mut body = Some(std::mem::take(request.body_mut()));
  let method = request.method().clone();
  let mut forward_headers = request.headers().clone();
  let mut uri = request.uri().clone();
  for redirect_count in 0..=MAX_REDIRECTS {
    ensure_allowed(&uri, &allowed_hosts)?;
    let mut builder = client.request(method.clone(), uri.to_string());
    if !same_origin(&original_origin, &uri) {
      forward_headers.remove(http::header::AUTHORIZATION);
    }
    builder = builder.headers(forward_headers.clone());
    if let Some(body) = body.take() {
      let stream = body
        .into_data_stream()
        .map_err(|error| io::Error::other(error.to_string()));
      builder = builder.body(reqwest::Body::wrap_stream(stream));
    }
    let response = builder.send().await?;
    if is_redirect(response.status()) {
      if !matches!(method, Method::GET | Method::HEAD) {
        return Err(io::Error::other("redirect on non-idempotent OCI request").into());
      }
      if redirect_count == MAX_REDIRECTS {
        return Err(io::Error::other("too many OCI redirects").into());
      }
      let location = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| io::Error::other("redirect is missing a valid location"))?;
      uri = response.url().join(location)?.as_str().parse()?;
      continue;
    }
    return response_to_transport(response).await;
  }
  Err(io::Error::other("too many OCI redirects").into())
}

async fn response_to_transport(
  response: reqwest::Response,
) -> Result<Response<Body>, transport::BoxError> {
  let status = StatusCode::from_u16(response.status().as_u16())?;
  let headers: HeaderMap = response.headers().clone();
  let stream = response.bytes_stream().scan(0usize, |total, part| {
    let result = match part {
      Ok(bytes) if total.saturating_add(bytes.len()) <= MAX_RESPONSE_BYTES => {
        *total += bytes.len();
        Ok(bytes)
      }
      Ok(_) => Err(io::Error::other("OCI metadata exceeds size limit")),
      Err(error) => Err(io::Error::other(error.to_string())),
    };
    std::future::ready(Some(result))
  });
  let frames = stream.map(|part| part.map(http_body::Frame::data));
  let body = Body::wrap(http_body_util::StreamBody::new(frames));
  let mut result = Response::builder().status(status).body(body)?;
  *result.headers_mut() = headers;
  Ok(result)
}

fn ensure_allowed(uri: &Uri, allowed_hosts: &BTreeSet<String>) -> io::Result<()> {
  let Some(host) = uri.host() else {
    return Err(io::Error::other("OCI request URI has no host"));
  };
  if uri.scheme_str() != Some("https") {
    return Err(io::Error::other("OCI request attempted insecure transport"));
  }
  let full_url = reqwest::Url::parse(&uri.to_string())
    .map_err(|_| io::Error::other("OCI request URI is invalid"))?;
  if !full_url.username().is_empty() || full_url.password().is_some() {
    return Err(io::Error::other(
      "OCI request URI contains user information",
    ));
  }
  let authority = match uri.port_u16() {
    Some(443) | None => host.to_ascii_lowercase(),
    Some(port) => format!("{}:{port}", host.to_ascii_lowercase()),
  };
  if !allowed_hosts.contains(&authority) {
    return Err(io::Error::other("OCI request host is not allowlisted"));
  }
  Ok(())
}

fn same_origin(left: &Uri, right: &Uri) -> bool {
  left.scheme_str() == right.scheme_str()
    && left.host().map(str::to_ascii_lowercase) == right.host().map(str::to_ascii_lowercase)
    && left.port_u16().unwrap_or(443) == right.port_u16().unwrap_or(443)
}

fn is_redirect(status: StatusCode) -> bool {
  matches!(
    status,
    StatusCode::MOVED_PERMANENTLY
      | StatusCode::FOUND
      | StatusCode::SEE_OTHER
      | StatusCode::TEMPORARY_REDIRECT
      | StatusCode::PERMANENT_REDIRECT
  )
}
