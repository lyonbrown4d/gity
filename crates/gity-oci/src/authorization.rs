use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Fully parsed request context passed to the platform's existing ACL adapter.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OciRequest {
  /// Public repository path (namespace/name); adapters resolve it to the
  /// stable project ID used as the internal storage namespace.
  pub repository: String,
  pub method: String,
  pub operation: OciOperation,
  /// Set only for cross-repository blob mounts.
  pub mount_source: Option<String>,
  /// Reference on manifest routes (tag or digest), if present.
  pub reference: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OciOperation {
  Probe,
  Pull,
  Push,
  Delete,
  Catalog,
}

#[async_trait]
pub trait OciAuthorization: Send + Sync + 'static {
  /// Validate the opaque Gity credential and enforce local project ACLs.
  /// Implementations must not trust identity headers from the client.
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError>;
}

/// Stable storage paths selected by the platform adapter after ACL checks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OciAuthorizationDecision {
  pub storage_repository: String,
  /// None on a mount request means start an ordinary upload: the gateway must
  /// remove both mount/from, including when the requested source is inaccessible.
  pub mount_source_storage_repository: Option<String>,
  /// Gity-managed mirror source chosen by the platform after ACL checks.
  pub mirror_source: Option<MirrorSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MirrorSource {
  pub registry_alias: String,
  pub repository: String,
  /// Present on a manifest route; blob and tags/list requests have no
  /// reference segment.
  pub reference: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
  Unauthenticated,
  Forbidden,
  NotFound,
  Unavailable,
}
