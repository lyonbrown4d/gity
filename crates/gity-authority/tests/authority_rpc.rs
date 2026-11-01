use async_trait::async_trait;
use fluxdi::Application;
use gity_authority::{AuthorityHealth, AuthorityModule, AuthoritySettings, RemoteOciAuthorization};
use gity_git_wire::NodeEndpoint;
use gity_oci::{
  AuthorizationError, MirrorSource, OciAuthorization, OciAuthorizationDecision, OciOperation,
  OciRequest,
};
use std::{
  net::SocketAddr,
  sync::{Arc, Mutex},
  time::Duration,
};

type RecordedRequest = Arc<Mutex<Option<(OciRequest, Option<String>)>>>;

struct RecordingAuthorization {
  received: RecordedRequest,
  error: Option<AuthorizationError>,
}

#[async_trait]
impl OciAuthorization for RecordingAuthorization {
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    *self.received.lock().unwrap() = Some((request.clone(), authorization.map(str::to_owned)));
    if let Some(error) = self.error {
      return Err(error);
    }
    Ok(OciAuthorizationDecision {
      storage_repository: "project-12/image".into(),
      mount_source_storage_repository: None,
      mirror_source: Some(MirrorSource {
        registry_alias: "hub".into(),
        repository: "library/alpine".into(),
        reference: request.reference.clone(),
      }),
    })
  }
}

#[tokio::test]
async fn rpc_forwards_opaque_user_authorization_and_shuts_down_cleanly() {
  let expected_peer_token = "p".repeat(32);
  let received = Arc::new(Mutex::new(None));
  let delegate = Arc::new(RecordingAuthorization {
    received: received.clone(),
    error: None,
  });
  let mut app = application(delegate, &expected_peer_token).await;
  let health = app
    .injector()
    .try_resolve::<AuthorityHealth>()
    .expect("health provider registered");
  assert!(health.is_healthy());

  let client = RemoteOciAuthorization::new(
    endpoint(health.bound_address().await.unwrap(), &expected_peer_token),
    Duration::from_secs(5),
  )
  .unwrap();
  let request = OciRequest {
    repository: "team/repository".into(),
    method: "GET".into(),
    operation: OciOperation::Pull,
    mount_source: None,
    reference: Some("v1.2.3".into()),
  };
  let decision = client
    .authorize(&request, Some("Bearer opaque-gity-user-secret"))
    .await
    .unwrap();
  assert_eq!(decision.storage_repository, "project-12/image");
  let received = received.lock().unwrap().clone().unwrap();
  assert_eq!(received.0, request);
  assert_eq!(
    received.1.as_deref(),
    Some("Bearer opaque-gity-user-secret")
  );

  app.shutdown().await.unwrap();
  assert!(!health.is_healthy());
}

#[tokio::test]
async fn peer_token_is_checked_and_authorization_errors_keep_status_meaning() {
  let server_token = "s".repeat(32);
  let received = Arc::new(Mutex::new(None));
  let delegate = Arc::new(RecordingAuthorization {
    received: received.clone(),
    error: Some(AuthorizationError::Forbidden),
  });
  let mut app = application(delegate, &server_token).await;
  let address = app
    .injector()
    .try_resolve::<AuthorityHealth>()
    .unwrap()
    .bound_address()
    .await
    .unwrap();
  let request = OciRequest {
    repository: "team/repository".into(),
    method: "GET".into(),
    operation: OciOperation::Pull,
    mount_source: None,
    reference: None,
  };
  let wrong_peer =
    RemoteOciAuthorization::new(endpoint(address, &"x".repeat(32)), Duration::from_secs(5))
      .unwrap();
  assert_eq!(
    wrong_peer.authorize(&request, None).await.unwrap_err(),
    AuthorizationError::Unauthenticated
  );
  assert!(received.lock().unwrap().is_none());

  let peer =
    RemoteOciAuthorization::new(endpoint(address, &server_token), Duration::from_secs(5)).unwrap();
  assert_eq!(
    peer.authorize(&request, None).await.unwrap_err(),
    AuthorizationError::Forbidden
  );
  assert_eq!(received.lock().unwrap().as_ref().unwrap().0, request);
  app.shutdown().await.unwrap();
}

#[test]
fn authority_requires_long_peer_tokens_and_mutual_tls_off_loopback() {
  let short_token = AuthoritySettings {
    address: "127.0.0.1:9102".parse().unwrap(),
    peer_token: "short".into(),
    tls_certificate: None,
    tls_key: None,
    client_ca: None,
  };
  assert!(short_token.validate().is_err());

  let remote_without_tls = AuthoritySettings {
    address: "192.0.2.10:9102".parse().unwrap(),
    peer_token: "p".repeat(32),
    tls_certificate: None,
    tls_key: None,
    client_ca: None,
  };
  assert!(remote_without_tls.validate().is_err());

  let endpoint = NodeEndpoint {
    id: "authority-main".into(),
    uri: "https://authority.internal:9102".into(),
    token: "p".repeat(32),
    ca_certificate: None,
    client_certificate: None,
    client_key: None,
  };
  assert!(RemoteOciAuthorization::new(endpoint, Duration::from_secs(1)).is_err());
}

async fn application(authorization: Arc<dyn OciAuthorization>, peer_token: &str) -> Application {
  let address = free_address().await;
  let module = AuthorityModule::new(
    AuthoritySettings {
      address,
      peer_token: peer_token.to_owned(),
      tls_certificate: None,
      tls_key: None,
      client_ca: None,
    },
    authorization,
  )
  .unwrap();
  let mut app = Application::new(module);
  app.bootstrap().await.unwrap();
  app
}

async fn free_address() -> SocketAddr {
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  drop(listener);
  address
}

fn endpoint(address: SocketAddr, token: &str) -> NodeEndpoint {
  NodeEndpoint {
    id: "authority-main".into(),
    uri: format!("http://{address}"),
    token: token.into(),
    ca_certificate: None,
    client_certificate: None,
    client_key: None,
  }
}
