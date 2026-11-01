use async_trait::async_trait;
use axum::http::{HeaderMap, StatusCode};
use fluxdi::Application;
use gity_authority::{
  AuthorityHealth, AuthorityModule, AuthoritySettings, RemotePackageAuthorization,
};
use gity_core::AppError;
use gity_git_wire::NodeEndpoint;
use gity_oci::{AuthorizationError, OciAuthorization, OciAuthorizationDecision, OciRequest};
use gity_package_registry::{PackageAction, PackageActor, PackageAuthorization};
use std::{
  sync::{Arc, Mutex},
  time::Duration,
};

struct NoOci;
#[async_trait]
impl OciAuthorization for NoOci {
  async fn authorize(
    &self,
    _: &OciRequest,
    _: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    Err(AuthorizationError::Forbidden)
  }
}
struct PackageAuthority(Arc<Mutex<Option<(i64, PackageAction, HeaderMap)>>>);
#[async_trait]
impl PackageAuthorization for PackageAuthority {
  async fn authorize(
    &self,
    project: i64,
    action: PackageAction,
    headers: &HeaderMap,
  ) -> Result<PackageActor, AppError> {
    *self.0.lock().unwrap() = Some((project, action, headers.clone()));
    if headers
      .get("authorization")
      .is_none_or(|value| value != "Bearer accepted-test-token")
    {
      return Err(AppError::forbidden("rejected test credential"));
    }
    Ok(PackageActor {
      user_id: Some(123),
      source: "manual".into(),
      pipeline_id: None,
      job_id: None,
      project_public: false,
    })
  }
}
fn endpoint(address: std::net::SocketAddr, token: &str) -> NodeEndpoint {
  NodeEndpoint {
    id: "main".into(),
    uri: format!("http://{address}"),
    token: token.into(),
    ca_certificate: None,
    client_certificate: None,
    client_key: None,
  }
}
#[tokio::test]
async fn package_authority_forwards_only_credentials_and_fails_closed_on_outage() {
  let token = "package-rpc-test-internal-secret-at-least-32-bytes";
  let received = Arc::new(Mutex::new(None));
  let module = AuthorityModule::new(
    AuthoritySettings {
      address: "127.0.0.1:0".parse().unwrap(),
      peer_token: token.into(),
      tls_certificate: None,
      tls_key: None,
      client_ca: None,
    },
    Arc::new(NoOci),
  )
  .unwrap()
  .with_packages(Arc::new(PackageAuthority(received.clone())));
  let mut app = Application::new(module);
  app.bootstrap().await.unwrap();
  let health = app.injector().try_resolve::<AuthorityHealth>().unwrap();
  let address = health.bound_address().await.unwrap();
  let remote =
    RemotePackageAuthorization::new(endpoint(address, token), Duration::from_secs(2)).unwrap();
  let mut headers = HeaderMap::new();
  headers.insert(
    "authorization",
    "Bearer accepted-test-token".parse().unwrap(),
  );
  headers.insert("x-gity-user-id", "untrusted-admin".parse().unwrap());
  let actor = remote
    .authorize(42, PackageAction::Write, &headers)
    .await
    .unwrap();
  assert_eq!(actor.user_id, Some(123));
  assert!(!actor.project_public);
  {
    let received = received.lock().unwrap();
    let (project, action, headers) = received.as_ref().unwrap();
    assert_eq!(*project, 42);
    assert!(matches!(action, PackageAction::Write));
    assert_eq!(headers.len(), 1);
    assert!(!headers.contains_key("x-gity-user-id"));
  }
  let denied = remote
    .authorize(42, PackageAction::Read, &HeaderMap::new())
    .await
    .unwrap_err();
  assert_eq!(denied.status, StatusCode::FORBIDDEN);
  let rogue = RemotePackageAuthorization::new(
    endpoint(address, "wrong-internal-peer-credential-at-least-32-bytes"),
    Duration::from_secs(2),
  )
  .unwrap();
  assert_eq!(
    rogue
      .authorize(42, PackageAction::Read, &headers)
      .await
      .unwrap_err()
      .status,
    StatusCode::UNAUTHORIZED
  );
  app.shutdown().await.unwrap();
  assert_eq!(
    remote
      .authorize(42, PackageAction::Read, &headers)
      .await
      .unwrap_err()
      .status,
    StatusCode::SERVICE_UNAVAILABLE
  );
}
