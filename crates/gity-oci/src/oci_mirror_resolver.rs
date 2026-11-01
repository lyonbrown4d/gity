use crate::{
  MirrorCredential, MirrorCredentialProvider, MirrorRegistry, MirrorResolveError, MirrorResolver,
  MirrorSettings, MirrorSource, OciSettings, mirror_transport::build_transport,
};
use async_trait::async_trait;
use oci_client::{Reference, secrets::RegistryAuth};
use std::{str::FromStr, sync::Arc};

const MANIFEST_TYPES: &[&str] = &[
  "application/vnd.oci.image.manifest.v1+json",
  "application/vnd.oci.image.index.v1+json",
  "application/vnd.docker.distribution.manifest.v2+json",
  "application/vnd.docker.distribution.manifest.list.v2+json",
];

#[cfg(test)]
#[path = "oci_mirror_resolver_tests.rs"]
mod tests;

pub struct OciClientMirrorResolver {
  mirror: MirrorSettings,
  timeout: std::time::Duration,
  credentials: Arc<dyn MirrorCredentialProvider>,
}

impl OciClientMirrorResolver {
  pub fn new(
    settings: &OciSettings,
    credentials: Arc<dyn MirrorCredentialProvider>,
  ) -> Result<Self, String> {
    settings.validate()?;
    let mirror = settings
      .mirror
      .clone()
      .ok_or_else(|| "OCI mirror settings are required".to_owned())?;
    Ok(Self {
      mirror,
      timeout: settings.request_timeout,
      credentials,
    })
  }

  async fn resolve_source(
    &self,
    source: &MirrorSource,
    registry: &MirrorRegistry,
  ) -> Result<String, MirrorResolveError> {
    let source_reference = source
      .reference
      .as_deref()
      .ok_or(MirrorResolveError::InvalidResponse)?;
    let reference_text = format!("{}/{}:{source_reference}", registry.host, source.repository);
    let reference =
      Reference::from_str(&reference_text).map_err(|_| MirrorResolveError::InvalidResponse)?;
    let transport = build_transport(self.mirror.allowed_hosts.clone(), self.timeout)
      .map_err(|_| MirrorResolveError::Unavailable)?;
    let auth = match registry.credential_secret_ref.as_deref() {
      Some(secret_ref) => match self.credentials.resolve_basic(secret_ref).await? {
        Some(MirrorCredential { username, password }) => RegistryAuth::Basic(username, password),
        None => return Err(MirrorResolveError::Unauthenticated),
      },
      None => RegistryAuth::Anonymous,
    };
    let (_, digest) = transport
      .pull_manifest_raw(&reference, &auth, MANIFEST_TYPES)
      .await
      .map_err(classify_oci_error)?;
    if !is_sha256_digest(&digest) {
      return Err(MirrorResolveError::InvalidResponse);
    }
    Ok(digest)
  }
}

#[async_trait]
impl MirrorResolver for OciClientMirrorResolver {
  async fn resolve_digest(&self, source: &MirrorSource) -> Result<String, MirrorResolveError> {
    let registry = self
      .mirror
      .registries
      .get(&source.registry_alias)
      .ok_or(MirrorResolveError::Forbidden)?;
    if !valid_repository(&source.repository)
      || !source.reference.as_deref().is_some_and(valid_reference)
    {
      return Err(MirrorResolveError::InvalidResponse);
    }
    self.resolve_source(source, registry).await
  }
}

fn classify_oci_error(error: oci_client::errors::OciDistributionError) -> MirrorResolveError {
  use oci_client::errors::OciDistributionError as OciError;
  match error {
    OciError::UnauthorizedError { .. } | OciError::AuthenticationFailure(_) => {
      MirrorResolveError::Unauthenticated
    }
    OciError::ServerError { code: 401, .. } => MirrorResolveError::Unauthenticated,
    OciError::ServerError { code: 403, .. } => MirrorResolveError::Forbidden,
    OciError::RegistryError { envelope, .. }
      if envelope
        .errors
        .iter()
        .any(|error| error.code == oci_client::errors::OciErrorCode::Unauthorized) =>
    {
      MirrorResolveError::Unauthenticated
    }
    OciError::RegistryError { envelope, .. }
      if envelope
        .errors
        .iter()
        .any(|error| error.code == oci_client::errors::OciErrorCode::Denied) =>
    {
      MirrorResolveError::Forbidden
    }
    _ => MirrorResolveError::Unavailable,
  }
}

fn valid_repository(repository: &str) -> bool {
  !repository.is_empty()
    && repository.split('/').all(|part| {
      !part.is_empty()
        && part
          .bytes()
          .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    })
}

fn valid_reference(reference: &str) -> bool {
  !reference.is_empty()
    && reference.len() <= 255
    && reference
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn is_sha256_digest(digest: &str) -> bool {
  digest
    .strip_prefix("sha256:")
    .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
