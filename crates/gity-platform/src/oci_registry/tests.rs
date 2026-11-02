use super::PlatformOciAuthorizer;
use base64::{Engine, engine::general_purpose::STANDARD};
use gity_oci::{AuthorizationError, OciAuthorization, OciOperation, OciRequest};

#[tokio::test]
async fn docker_probe_requires_verified_credentials_and_preserves_repository_acl() {
  let (_dir, state) = crate::features::test_support::fixture().await;
  let authorizer = PlatformOciAuthorizer::new(state);
  let mut request = OciRequest {
    repository: String::new(),
    method: "GET".into(),
    operation: OciOperation::Probe,
    mount_source: None,
    reference: None,
  };
  for credentials in [None, Some("Basic invalid"), Some("Bearer invalid")] {
    assert!(matches!(
      authorizer.authorize(&request, credentials).await,
      Err(AuthorizationError::Unauthenticated)
    ));
  }
  let credentials = format!("Basic {}", STANDARD.encode("alice:alice"));
  assert!(
    authorizer
      .authorize(&request, Some(&credentials))
      .await
      .is_ok()
  );
  let outsider = format!("Basic {}", STANDARD.encode("outsider:outsider"));
  assert!(
    authorizer
      .authorize(&request, Some(&outsider))
      .await
      .is_ok()
  );
  request.repository = "team/p20/image".into();
  request.operation = OciOperation::Push;
  request.method = "PUT".into();
  request.reference = Some("latest".into());
  assert!(matches!(
    authorizer.authorize(&request, Some(&outsider)).await,
    Err(AuthorizationError::Forbidden)
  ));
  let decision = authorizer
    .authorize(&request, Some(&credentials))
    .await
    .unwrap();
  assert_eq!(
    decision.storage_repository,
    "projects/20/repositories/image"
  );
}
