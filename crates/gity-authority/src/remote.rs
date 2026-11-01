use async_trait::async_trait;
use gity_git_wire::{
  NodeClient, NodeEndpoint,
  identity_wire::{AuthorizeOciRequest, identity_authority_client::IdentityAuthorityClient},
};
use gity_oci::{AuthorizationError, OciAuthorization, OciAuthorizationDecision, OciRequest};
use std::time::Duration;
use tonic::Code;

const MAX_JSON_BYTES: usize = 1024 * 1024;

pub struct RemoteOciAuthorization {
  node: NodeClient,
  request_timeout: Duration,
}

impl RemoteOciAuthorization {
  pub fn new(endpoint: NodeEndpoint, request_timeout: Duration) -> Result<Self, String> {
    if request_timeout.is_zero() {
      return Err("authority request timeout must be positive".into());
    }
    validate_endpoint(&endpoint)?;
    let node = NodeClient::new(endpoint).map_err(|_| "invalid authority endpoint".to_owned())?;
    Ok(Self {
      node,
      request_timeout,
    })
  }
}

pub(crate) fn validate_endpoint(endpoint: &NodeEndpoint) -> Result<(), String> {
  if endpoint.token.len() < 32 {
    return Err("authority peer token must be at least 32 bytes".into());
  }
  let url = url::Url::parse(&endpoint.uri).map_err(|_| "invalid authority URI".to_owned())?;
  let host = url
    .host_str()
    .ok_or_else(|| "authority host is required".to_owned())?;
  let loopback = host == "localhost"
    || host
      .parse::<std::net::IpAddr>()
      .is_ok_and(|ip| ip.is_loopback());
  if !loopback
    && (url.scheme() != "https"
      || endpoint.ca_certificate.is_none()
      || endpoint.client_certificate.is_none()
      || endpoint.client_key.is_none())
  {
    return Err("remote authority requires trusted mutual TLS".into());
  }
  Ok(())
}

#[async_trait]
impl OciAuthorization for RemoteOciAuthorization {
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    let request_json = serde_json::to_vec(request).map_err(|_| AuthorizationError::Unavailable)?;
    if request_json.len() > MAX_JSON_BYTES {
      return Err(AuthorizationError::Unavailable);
    }
    let message = AuthorizeOciRequest {
      request_json,
      authorization: authorization.map(str::to_owned),
    };
    let request = self
      .node
      .authenticate(message)
      .map_err(|_| AuthorizationError::Unavailable)?;
    let channel = self.node.channel();
    let mut client =
      IdentityAuthorityClient::new(channel).max_decoding_message_size(MAX_JSON_BYTES + 1024);
    let rpc = client.authorize_oci(request);
    let response = tokio::time::timeout(self.request_timeout, rpc)
      .await
      .map_err(|_| AuthorizationError::Unavailable)?
      .map_err(map_status)?
      .into_inner();
    if response.decision_json.len() > MAX_JSON_BYTES {
      return Err(AuthorizationError::Unavailable);
    }
    serde_json::from_slice(&response.decision_json).map_err(|_| AuthorizationError::Unavailable)
  }
}

fn map_status(status: tonic::Status) -> AuthorizationError {
  match status.code() {
    Code::Unauthenticated => AuthorizationError::Unauthenticated,
    Code::PermissionDenied => AuthorizationError::Forbidden,
    Code::NotFound => AuthorizationError::NotFound,
    Code::Unavailable | Code::DeadlineExceeded | Code::Cancelled => AuthorizationError::Unavailable,
    _ => AuthorizationError::Unavailable,
  }
}
