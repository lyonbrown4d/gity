use crate::{MirrorSource, OciSettings};
use async_trait::async_trait;

/// A secret resolved from the platform's configured secret store. Keep this
/// type out of request/response models and do not derive `Debug` for it.
pub struct MirrorCredential {
  pub username: String,
  pub password: String,
}

#[async_trait]
pub trait MirrorCredentialProvider: Send + Sync + 'static {
  async fn resolve_basic(
    &self,
    secret_ref: &str,
  ) -> Result<Option<MirrorCredential>, MirrorResolveError>;
}

/// Resolve mutable references before asking Angos for a cache entry. A
/// successful result is a validated content digest, never a stale tag value.
#[async_trait]
pub trait MirrorResolver: Send + Sync + 'static {
  async fn resolve_digest(&self, source: &MirrorSource) -> Result<String, MirrorResolveError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MirrorResolveError {
  Unauthenticated,
  Forbidden,
  Unavailable,
  InvalidResponse,
}

pub(crate) fn resolver_is_configured(settings: &OciSettings, source: &MirrorSource) -> bool {
  settings
    .mirror
    .as_ref()
    .is_some_and(|mirror| mirror.registries.contains_key(&source.registry_alias))
}
