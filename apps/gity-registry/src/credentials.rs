use super::settings::CredentialEnvironment;
use gity_oci::MirrorCredentialProvider;
use std::collections::BTreeMap;
pub(super) struct EnvironmentCredentials(pub(super) BTreeMap<String, CredentialEnvironment>);

#[async_trait::async_trait]
impl MirrorCredentialProvider for EnvironmentCredentials {
  async fn resolve_basic(
    &self,
    secret_ref: &str,
  ) -> Result<Option<gity_oci::MirrorCredential>, gity_oci::MirrorResolveError> {
    let Some(reference) = self.0.get(secret_ref) else {
      return Err(gity_oci::MirrorResolveError::Unavailable);
    };
    let username =
      std::env::var(&reference.username).map_err(|_| gity_oci::MirrorResolveError::Unavailable)?;
    let password =
      std::env::var(&reference.password).map_err(|_| gity_oci::MirrorResolveError::Unavailable)?;
    if username.is_empty() || password.is_empty() {
      return Err(gity_oci::MirrorResolveError::Unavailable);
    }
    Ok(Some(gity_oci::MirrorCredential { username, password }))
  }
}
