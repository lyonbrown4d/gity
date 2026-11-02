//! Explicit opt-in protocol experiment; never uses an existing registry or config.
use super::*;
use crate::{OciAuthorizationDecision, OciOperation, UpstreamPolicy};
use async_trait::async_trait;
use axum::body::to_bytes;
use std::{
  path::Path,
  process::Stdio,
  sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};
use tokio::io::AsyncWriteExt;

struct Credentials {
  anonymous_probe: bool,
  requests: Mutex<Vec<(OciOperation, bool)>>,
}
#[async_trait]
impl OciAuthorization for Credentials {
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    let valid = authorization == Some("Basic cHJvYmU6b3BhcXVlLXRlc3QtdG9rZW4=");
    self
      .requests
      .lock()
      .unwrap()
      .push((request.operation, valid));
    if !valid
      && !(self.anonymous_probe
        && request.operation == OciOperation::Probe
        && authorization.is_none())
    {
      return Err(AuthorizationError::Unauthenticated);
    }
    Ok(OciAuthorizationDecision {
      storage_repository: request.repository.clone(),
      mount_source_storage_repository: None,
      mirror_source: None,
    })
  }
}

async fn engine(State(manifests): State<Arc<AtomicUsize>>, request: Request<Body>) -> Response {
  assert!(!request.headers().contains_key("authorization"));
  let path = request.uri().path();
  if path == "/v2/" {
    return ([("docker-distribution-api-version", "registry/2.0")], "{}").into_response();
  }
  // Cached blobs make the experiment independent of layer-upload mechanics.
  if request.method() == Method::HEAD && path.contains("/blobs/sha256:") {
    let digest = path.rsplit('/').next().unwrap();
    return Response::builder()
      .status(200)
      .header("docker-content-digest", digest)
      .body(Body::empty())
      .unwrap();
  }
  if request.method() == Method::PUT && path.contains("/manifests/") {
    let body = to_bytes(request.into_body(), 1024 * 1024).await.unwrap();
    let digest = oci_client::client::Config::oci_v1(body, None).sha256_digest();
    manifests.fetch_add(1, Ordering::SeqCst);
    return Response::builder()
      .status(201)
      .header("docker-content-digest", digest)
      .body(Body::empty())
      .unwrap();
  }
  (
    StatusCode::NOT_FOUND,
    axum::Json(serde_json::json!({"errors":[{"code":"MANIFEST_UNKNOWN","message":"not cached"}]})),
  )
    .into_response()
}

async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap().to_string();
  let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  (address, task)
}

async fn docker(config: &Path, arguments: &[&str], input: Option<&[u8]>) -> std::process::Output {
  let mut command = tokio::process::Command::new("docker");
  command
    .arg("--config")
    .arg(config)
    .args(arguments)
    .kill_on_drop(true)
    .stdin(if input.is_some() {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  let mut child = command.spawn().unwrap();
  if let Some(input) = input {
    child.stdin.take().unwrap().write_all(input).await.unwrap();
  }
  tokio::time::timeout(Duration::from_secs(90), child.wait_with_output())
    .await
    .unwrap()
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires local Docker daemon and cached alpine:3.22; creates/removes only unique test tags and /tmp config"]
async fn real_docker_requires_authenticated_ping_for_login_and_challenge_for_push() {
  for authenticated_probe in [false, true] {
    let manifests = Arc::new(AtomicUsize::new(0));
    let (upstream, engine_task) =
      serve(Router::new().fallback(engine).with_state(manifests.clone())).await;
    let authorizer = Arc::new(Credentials {
      anonymous_probe: !authenticated_probe,
      requests: Mutex::new(Vec::new()),
    });
    let gateway = OciGateway::new(
      OciSettings {
        upstream: format!("http://{upstream}"),
        request_timeout: Duration::from_secs(10),
        upstream_policy: UpstreamPolicy::default(),
        mirror: None,
      },
      authorizer.clone(),
    )
    .unwrap();
    let (address, gateway_task) = serve(router(gateway)).await;
    let config = std::env::temp_dir().join(format!(
      "gity-docker-handshake-{}-{}",
      std::process::id(),
      address.replace(':', "-")
    ));
    std::fs::create_dir(&config).unwrap();
    let image = format!("{address}/gity-handshake/image:probe");
    let wrong_login = docker(
      &config,
      &["login", &address, "--username", "probe", "--password-stdin"],
      Some(b"incorrect\n"),
    )
    .await;
    let login = docker(
      &config,
      &["login", &address, "--username", "probe", "--password-stdin"],
      Some(b"opaque-test-token\n"),
    )
    .await;
    let tagged = docker(&config, &["tag", "alpine:3.22", &image], None).await;
    let pushed = docker(&config, &["push", &image], None).await;
    let removed = docker(&config, &["image", "rm", &image], None).await;
    gateway_task.abort();
    engine_task.abort();
    std::fs::remove_dir_all(&config).unwrap();
    let requests = authorizer.requests.lock().unwrap();
    let authenticated_repository_requests = requests
      .iter()
      .filter(|(operation, valid)| *operation != OciOperation::Probe && *valid)
      .count();
    eprintln!(
      "authenticated_probe={authenticated_probe} wrong_login={} login={} push={} authenticated_repository_requests={} manifests={}",
      wrong_login.status.success(),
      login.status.success(),
      pushed.status.success(),
      authenticated_repository_requests,
      manifests.load(Ordering::SeqCst)
    );
    if !pushed.status.success() {
      eprintln!("push error: {}", String::from_utf8_lossy(&pushed.stderr));
    }
    assert!(tagged.status.success() && removed.status.success());
    assert!(
      login.status.success(),
      "{}",
      String::from_utf8_lossy(&login.stderr)
    );
    assert!(pushed.status.success());
    assert!(authenticated_repository_requests > 0 && manifests.load(Ordering::SeqCst) > 0);
    if authenticated_probe {
      assert!(!wrong_login.status.success());
    } else {
      assert!(
        wrong_login.status.success(),
        "anonymous ping makes invalid login look successful"
      );
      // Repository challenges allow push to retry, but cannot validate login.
    }
  }
}
