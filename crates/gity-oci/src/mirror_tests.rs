use super::*;
use crate::{
  MirrorResolveError, MirrorSettings, MirrorSource, OciAuthorizationDecision, OciRequest,
  UpstreamPolicy,
};
use async_trait::async_trait;
use axum::{Router, extract::State, http::StatusCode, routing::any};
use std::{
  collections::{BTreeMap, BTreeSet, VecDeque},
  net::SocketAddr,
  sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

#[tokio::test]
async fn mutable_tags_resolve_before_cache_and_failures_never_use_stale_cache() {
  let (upstream, upstream_calls, upstream_task) = start_fake_angos().await;
  let resolver = Arc::new(FakeResolver::new([
    Ok(format!("sha256:{}", "a".repeat(64))),
    Ok(format!("sha256:{}", "b".repeat(64))),
    Err(MirrorResolveError::Unauthenticated),
    Err(MirrorResolveError::Forbidden),
    Err(MirrorResolveError::NotFound),
  ]));
  let mut gateway_settings = settings(Some(mirror_config()));
  gateway_settings.upstream = format!("http://{upstream}");
  let gateway = OciGateway::new_with_mirror(
    gateway_settings,
    Arc::new(FakeAuthorizer),
    Some(resolver.clone()),
  )
  .unwrap();
  let (address, gateway_task) = start_gateway(gateway).await;
  let client = reqwest::Client::new();

  for _ in 0..2 {
    let response = client
      .get(format!("http://{address}/v2/team/repo/manifests/latest"))
      .send()
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
  }
  for (error, status) in [
    (
      MirrorResolveError::Unauthenticated,
      StatusCode::UNAUTHORIZED,
    ),
    (MirrorResolveError::Forbidden, StatusCode::FORBIDDEN),
    (MirrorResolveError::NotFound, StatusCode::NOT_FOUND),
  ] {
    let response = client
      .get(format!("http://{address}/v2/team/repo/manifests/latest"))
      .send()
      .await
      .unwrap();
    assert_eq!(response.status(), status, "resolver error {error:?}");
  }
  let paths = upstream_calls.paths.lock().unwrap().clone();
  assert_eq!(paths.len(), 2);
  assert!(paths[0].ends_with(&format!("/manifests/sha256:{}", "a".repeat(64))));
  assert!(paths[1].ends_with(&format!("/manifests/sha256:{}", "b".repeat(64))));
  assert_eq!(upstream_calls.no_redirect.load(Ordering::SeqCst), 2);
  assert_eq!(resolver.calls.load(Ordering::SeqCst), 5);

  gateway_task.abort();
  upstream_task.abort();
}

#[derive(Clone)]
struct FakeAuthorizer;

#[async_trait]
impl OciAuthorization for FakeAuthorizer {
  async fn authorize(
    &self,
    request: &OciRequest,
    _authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    Ok(OciAuthorizationDecision {
      storage_repository: "project-7/repo".into(),
      mount_source_storage_repository: None,
      mirror_source: Some(MirrorSource {
        registry_alias: "upstream-one".into(),
        repository: "library/image".into(),
        reference: request.reference.clone(),
      }),
    })
  }
}

struct FakeResolver {
  calls: AtomicUsize,
  results: Mutex<VecDeque<Result<String, MirrorResolveError>>>,
}

impl FakeResolver {
  fn new(results: impl IntoIterator<Item = Result<String, MirrorResolveError>>) -> Self {
    Self {
      calls: AtomicUsize::new(0),
      results: Mutex::new(results.into_iter().collect()),
    }
  }
}

#[async_trait]
impl MirrorResolver for FakeResolver {
  async fn resolve_digest(&self, _source: &MirrorSource) -> Result<String, MirrorResolveError> {
    self.calls.fetch_add(1, Ordering::SeqCst);
    self.results.lock().unwrap().pop_front().unwrap()
  }
}

async fn start_fake_angos() -> (
  SocketAddr,
  Arc<FakeUpstreamCalls>,
  tokio::task::JoinHandle<()>,
) {
  let paths = Arc::new(FakeUpstreamCalls::default());
  let app = Router::new()
    .route("/v2/{*path}", any(record_path))
    .with_state(paths.clone());
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  (address, paths, task)
}

async fn record_path(
  State(paths): State<Arc<FakeUpstreamCalls>>,
  request: axum::http::Request<axum::body::Body>,
) -> StatusCode {
  paths
    .paths
    .lock()
    .unwrap()
    .push(request.uri().path().to_owned());
  if request
    .headers()
    .get("x-angos-no-redirect")
    .is_some_and(|value| value == "true")
  {
    paths.no_redirect.fetch_add(1, Ordering::SeqCst);
  }
  StatusCode::OK
}

#[derive(Default)]
struct FakeUpstreamCalls {
  paths: Mutex<Vec<String>>,
  no_redirect: AtomicUsize,
}

async fn start_gateway(gateway: OciGateway) -> (SocketAddr, tokio::task::JoinHandle<()>) {
  let app = router(gateway);
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  (address, task)
}

fn mirror_config() -> MirrorSettings {
  MirrorSettings {
    registries: BTreeMap::from([(
      "upstream-one".into(),
      crate::MirrorRegistry {
        host: "registry.example".into(),
        credential_secret_ref: None,
      },
    )]),
    allowed_hosts: BTreeSet::from(["registry.example".into()]),
  }
}

fn settings(mirror: Option<MirrorSettings>) -> OciSettings {
  OciSettings {
    upstream: "http://127.0.0.1:18000".into(),
    request_timeout: Duration::from_secs(30),
    upstream_policy: UpstreamPolicy {
      allowed_hosts: BTreeSet::from(["registry.example".into()]),
    },
    mirror,
  }
}

#[derive(Clone)]
struct MixedAuthorizer;

#[async_trait]
impl OciAuthorization for MixedAuthorizer {
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    let mut decision = FakeAuthorizer.authorize(request, authorization).await?;
    if request.repository == "team/hosted" {
      decision.mirror_source = None;
    }
    Ok(decision)
  }
}

#[tokio::test]
async fn configured_mirrors_do_not_disable_hosted_pushes_or_allow_mirror_mutations() {
  let (upstream, calls, upstream_task) = start_fake_angos().await;
  let mut settings = settings(Some(mirror_config()));
  settings.upstream = format!("http://{upstream}");
  let gateway = OciGateway::new(settings, Arc::new(MixedAuthorizer)).unwrap();
  let (address, gateway_task) = start_gateway(gateway).await;
  let client = reqwest::Client::new();
  for method in [reqwest::Method::POST, reqwest::Method::DELETE] {
    let suffix = if method == reqwest::Method::POST {
      "blobs/uploads/"
    } else {
      "manifests/sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    };
    for (repository, expected) in [
      ("hosted", StatusCode::OK),
      ("mirror", StatusCode::METHOD_NOT_ALLOWED),
    ] {
      let response = client
        .request(
          method.clone(),
          format!("http://{address}/v2/team/{repository}/{suffix}"),
        )
        .send()
        .await
        .unwrap();
      assert_eq!(response.status(), expected, "{method} {repository}");
    }
  }
  let paths = calls.paths.lock().unwrap().clone();
  assert_eq!(paths.len(), 2, "mirror mutations must never reach Angos");
  assert_eq!(paths[0], "/v2/project-7/repo/blobs/uploads/");
  assert!(paths[1].starts_with("/v2/project-7/repo/manifests/sha256:"));
  gateway_task.abort();
  upstream_task.abort();
}
