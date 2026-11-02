use super::*;
use crate::{OciAuthorizationDecision, OciOperation, UpstreamPolicy};
use async_trait::async_trait;
use axum::http::HeaderMap;
use std::{sync::Mutex, time::Duration};

#[derive(Default)]
struct ProbeAuthorizer {
  calls: Mutex<Vec<OciRequest>>,
}
#[async_trait]
impl OciAuthorization for ProbeAuthorizer {
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    self.calls.lock().unwrap().push(request.clone());
    if authorization != Some("Bearer probe-test-credential") {
      return Err(AuthorizationError::Unauthenticated);
    }
    Ok(OciAuthorizationDecision {
      storage_repository: String::new(),
      mount_source_storage_repository: None,
      mirror_source: None,
    })
  }
}

type Calls = Arc<Mutex<Vec<(Method, String, HeaderMap)>>>;

async fn upstream(State(calls): State<Calls>, request: Request<Body>) -> Response {
  calls.lock().unwrap().push((
    request.method().clone(),
    request.uri().to_string(),
    request.headers().clone(),
  ));
  let status = if request.uri().query() == Some("unavailable=1") {
    StatusCode::SERVICE_UNAVAILABLE
  } else {
    StatusCode::OK
  };
  (
    status,
    [("docker-distribution-api-version", "registry/2.0")],
    "upstream registry response",
  )
    .into_response()
}

async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let origin = format!("http://{}", listener.local_addr().unwrap());
  let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  (origin, task)
}

#[tokio::test]
async fn standard_v2_slash_probe_uses_real_router_authorization_and_upstream() {
  let calls: Calls = Arc::default();
  let mock = Router::new()
    .route("/v2", any(upstream))
    .route("/v2/", any(upstream))
    .with_state(calls.clone());
  let (origin, upstream_task) = serve(mock).await;
  let authorizer = Arc::new(ProbeAuthorizer::default());
  let gateway = OciGateway::new(
    OciSettings {
      upstream: origin,
      request_timeout: Duration::from_secs(5),
      upstream_policy: UpstreamPolicy::default(),
      mirror: None,
    },
    authorizer.clone(),
  )
  .unwrap();
  let (origin, gateway_task) = serve(router(gateway)).await;
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  for path in ["/v2", "/v2/"] {
    for method in [Method::GET, Method::HEAD] {
      let response = client
        .request(method.clone(), format!("{origin}{path}"))
        .bearer_auth("probe-test-credential")
        .header("x-gity-user-id", "forged")
        .send()
        .await
        .unwrap();
      assert_eq!(response.status(), StatusCode::OK, "{method} {path}");
      assert_eq!(
        response.headers()["docker-distribution-api-version"],
        "registry/2.0"
      );
      let body = response.text().await.unwrap();
      assert_eq!(
        body,
        if method == Method::HEAD {
          ""
        } else {
          "upstream registry response"
        }
      );
    }
  }
  let recorded = calls.lock().unwrap().clone();
  assert_eq!(recorded.len(), 4);
  for ((method, path, headers), (expected_method, expected_path)) in recorded.iter().zip([
    (Method::GET, "/v2"),
    (Method::HEAD, "/v2"),
    (Method::GET, "/v2/"),
    (Method::HEAD, "/v2/"),
  ]) {
    assert_eq!(method, expected_method);
    assert_eq!(path, expected_path);
    assert!(!headers.contains_key("authorization"));
    assert!(!headers.contains_key("x-gity-user-id"));
    assert_eq!(headers["x-angos-no-redirect"], "true");
  }
  for path in ["/v2", "/v2/"] {
    let response = client
      .get(format!("{origin}{path}"))
      .header("x-gity-user-id", "forged")
      .send()
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
      response.headers()[http::header::WWW_AUTHENTICATE],
      "Basic realm=\"Gity registry\""
    );
  }
  assert_eq!(
    calls.lock().unwrap().len(),
    4,
    "denied probes reached upstream"
  );
  let response = client
    .post(format!("{origin}/v2/"))
    .bearer_auth("probe-test-credential")
    .send()
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
  assert_eq!(calls.lock().unwrap().len(), 4);
  let response = client
    .get(format!("{origin}/v2/?unavailable=1"))
    .bearer_auth("probe-test-credential")
    .send()
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
  assert_eq!(response.text().await.unwrap(), "upstream registry response");
  let authorized = authorizer.calls.lock().unwrap();
  assert_eq!(authorized.len(), 7);
  assert!(
    authorized
      .iter()
      .all(|request| request.operation == OciOperation::Probe
        && request.repository.is_empty()
        && request.reference.is_none())
  );
  gateway_task.abort();
  upstream_task.abort();
}

#[tokio::test]
async fn readiness_probes_only_fixed_upstream_without_client_identity() {
  let calls: Calls = Arc::default();
  let mock = Router::new().fallback(upstream).with_state(calls.clone());
  let (upstream_origin, upstream_task) = serve(mock).await;
  let authorizer = Arc::new(ProbeAuthorizer::default());
  let gateway = OciGateway::new(
    OciSettings {
      upstream: upstream_origin,
      request_timeout: Duration::from_secs(1),
      upstream_policy: UpstreamPolicy::default(),
      mirror: None,
    },
    authorizer.clone(),
  )
  .unwrap();
  let (origin, gateway_task) = serve(router(gateway)).await;
  let client = reqwest::Client::new();
  let response = client
    .get(format!("{origin}/readyz?unavailable=1"))
    .bearer_auth("must-not-forward")
    .header("x-gity-user-id", "forged")
    .send()
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  assert!(response.bytes().await.unwrap().is_empty());
  let recorded = calls.lock().unwrap().clone();
  assert_eq!(recorded.len(), 1);
  assert_eq!(recorded[0].0, Method::GET);
  assert_eq!(recorded[0].1, "/v2/");
  assert!(!recorded[0].2.contains_key("authorization"));
  assert!(!recorded[0].2.contains_key("x-gity-user-id"));
  assert!(authorizer.calls.lock().unwrap().is_empty());
  upstream_task.abort();
  let _ = upstream_task.await;
  let response = client.get(format!("{origin}/readyz")).send().await.unwrap();
  assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
  gateway_task.abort();
}
