use super::{should_drop_header, target_uri};
use axum::http::{HeaderMap, HeaderName};

#[test]
fn proxy_keeps_path_query_and_opaque_auth_but_drops_forged_identity() {
  let headers = HeaderMap::new();
  assert_eq!(
    target_uri("https://gity-server", "/api/projects/42", Some("page=2")),
    "https://gity-server/api/projects/42?page=2"
  );
  assert!(!should_drop_header(
    &HeaderName::from_static("authorization"),
    &headers
  ));
  for name in [
    "x-gity-user-id",
    "x-authenticated-user",
    "x-forwarded-for",
    "forwarded",
    "content-length",
  ] {
    assert!(should_drop_header(&HeaderName::from_static(name), &headers));
  }
}

#[test]
fn proxy_drops_connection_named_headers() {
  let mut headers = HeaderMap::new();
  headers.insert("connection", "x-private-hop".parse().unwrap());
  assert!(should_drop_header(
    &HeaderName::from_static("x-private-hop"),
    &headers
  ));
}

#[test]
fn only_project_package_protocols_use_the_registry_route() {
  for path in [
    "/api/v1/projects/42/packages",
    "/api/v1/repos/42/packages",
    "/api/v1/projects/42/packages/cargo/index/config.json",
    "/api/v1/projects/42/packages/go/example.test/a/@v/v1.0.0.mod",
  ] {
    assert!(super::package_path(path), "{path}");
  }
  for path in [
    "/api/v1/projects/42/packages-invalid",
    "/api/v1/projects/42/repository/files/packages/a",
    "/api/v1/projects/invalid/packages",
    "/api/v1/users/42/packages",
  ] {
    assert!(!super::package_path(path), "{path}");
  }
}

#[tokio::test]
async fn package_route_preserves_public_host_and_replaces_forged_forwarding_headers() {
  use axum::{
    Router,
    body::{Body, to_bytes},
    http::Request,
    routing::get,
  };
  use tower::ServiceExt;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let stop = tokio_util::sync::CancellationToken::new();
  let shutdown = stop.clone();
  let path = "/api/v1/projects/42/packages/cargo/index/config.json";
  let app = Router::new().route(
    path,
    get(|headers: HeaderMap| async move {
      axum::Json(serde_json::json!({"host":headers["host"].to_str().unwrap(),
      "forwarded_host":headers["x-forwarded-host"].to_str().unwrap(),
      "scheme":headers["x-forwarded-proto"].to_str().unwrap(),
      "forged_identity":headers.contains_key("x-gity-user-id"),
      "opaque_auth":headers["authorization"].to_str().unwrap()}))
    }),
  );
  let server = tokio::spawn(async move {
    axum::serve(listener, app)
      .with_graceful_shutdown(shutdown.cancelled_owned())
      .await
      .unwrap();
  });
  let proxy = super::HttpProxy::new(
    "http://127.0.0.1:9".into(),
    std::time::Duration::from_secs(2),
    4,
    crate::GatewayHealth::default(),
  )
  .unwrap()
  .with_package_origin(Some(format!("http://{address}")))
  .with_public_scheme("https".into());
  let request = Request::builder()
    .uri(path)
    .header("host", "packages.example.test:8443")
    .header("authorization", "Bearer opaque-test-credential")
    .header("x-forwarded-proto", "invalid")
    .header("x-forwarded-host", "forged.example.test")
    .header("x-gity-user-id", "admin")
    .body(Body::empty())
    .unwrap();
  let response = super::router(proxy).oneshot(request).await.unwrap();
  assert_eq!(response.status(), 200);
  let body: serde_json::Value =
    serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
  assert_eq!(body["host"], "packages.example.test:8443");
  assert_eq!(body["forwarded_host"], "packages.example.test:8443");
  assert_eq!(body["scheme"], "https");
  assert_eq!(body["forged_identity"], false);
  assert_eq!(body["opaque_auth"], "Bearer opaque-test-credential");
  stop.cancel();
  server.await.unwrap();
}
