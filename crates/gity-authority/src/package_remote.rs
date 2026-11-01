use axum::http::{HeaderMap, StatusCode};
use gity_core::AppError;
use gity_git_wire::{
  NodeClient, NodeEndpoint,
  identity_wire::{AuthorizePackageRequest, identity_authority_client::IdentityAuthorityClient},
};
use gity_package_registry::{PackageAction, PackageActor, PackageAuthorization};
use std::time::Duration;

pub struct RemotePackageAuthorization {
  node: NodeClient,
  timeout: Duration,
}
impl RemotePackageAuthorization {
  pub fn new(endpoint: NodeEndpoint, timeout: Duration) -> Result<Self, String> {
    super::remote::validate_endpoint(&endpoint)?;
    if timeout.is_zero() {
      return Err("package authority timeout must be positive".into());
    }
    Ok(Self {
      node: NodeClient::new(endpoint)
        .map_err(|_| "invalid package authority endpoint".to_owned())?,
      timeout,
    })
  }
}
#[async_trait::async_trait]
impl PackageAuthorization for RemotePackageAuthorization {
  async fn authorize(
    &self,
    project_id: i64,
    action: PackageAction,
    headers: &HeaderMap,
  ) -> Result<PackageActor, AppError> {
    let mut credentials = std::collections::HashMap::new();
    let mut size = 0;
    for name in super::package_rpc::CREDENTIAL_HEADERS {
      if let Some(value) = headers.get(name) {
        let value = value
          .to_str()
          .map_err(|_| AppError::unauthorized("invalid package credential"))?;
        size += value.len();
        if size > 16384 {
          return Err(AppError::unauthorized("package credential is too large"));
        }
        credentials.insert(name.to_owned(), value.to_owned());
      }
    }
    let action = match action {
      PackageAction::Read => "read",
      PackageAction::Write => "write",
      PackageAction::Delete => "delete",
    }
    .to_owned();
    let request = self
      .node
      .authenticate(AuthorizePackageRequest {
        project_id,
        action,
        credentials,
      })
      .map_err(|_| unavailable())?;
    let mut client =
      IdentityAuthorityClient::new(self.node.channel()).max_decoding_message_size(65536);
    let response = tokio::time::timeout(self.timeout, client.authorize_package(request))
      .await
      .map_err(|_| unavailable())?
      .map_err(|status| match status.code() {
        tonic::Code::Unauthenticated => AppError::unauthorized("package credential rejected"),
        tonic::Code::PermissionDenied => AppError::forbidden("package operation forbidden"),
        tonic::Code::NotFound => AppError::not_found("package project not found"),
        tonic::Code::InvalidArgument => {
          AppError::bad_request("invalid package authorization request")
        }
        _ => unavailable(),
      })?
      .into_inner();
    serde_json::from_slice(&response.actor_json).map_err(|_| unavailable())
  }
}
fn unavailable() -> AppError {
  AppError::new(
    StatusCode::SERVICE_UNAVAILABLE,
    "package authority is unavailable",
  )
}
