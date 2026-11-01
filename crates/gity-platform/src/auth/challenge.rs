use axum::{
  extract::Request,
  http::{HeaderValue, StatusCode, header},
  middleware::Next,
  response::Response,
};

/// Native Git and registry clients negotiate Basic authentication. JSON API
/// errors use Bearer so browser fetch does not open a native password dialog.
pub(crate) async fn basic_challenge(request: Request, next: Next) -> Response {
  let mut response = next.run(request).await;
  if response.status() == StatusCode::UNAUTHORIZED {
    response.headers_mut().insert(
      header::WWW_AUTHENTICATE,
      HeaderValue::from_static("Basic realm=\"Gity\""),
    );
  }
  response
}
