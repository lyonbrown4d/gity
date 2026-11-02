use crate::AppState;
use gity_core::AppError;
use russh::keys::{HashAlg, PrivateKey, PublicKey, ssh_key::LineEnding};
use serde_json::Value;
use std::{io::Write, path::Path};

pub(crate) fn canonical_key(raw: &str) -> Result<(String, String), AppError> {
  let raw = raw.trim();
  if raw.len() > 16384 || raw.is_empty() || raw.contains(['\r', '\n', '\0']) {
    return Err(AppError::bad_request("SSH public key is invalid"));
  }
  let mut key =
    PublicKey::from_openssh(raw).map_err(|_| AppError::bad_request("SSH public key is invalid"))?;
  key.set_comment("");
  let fingerprint = key.fingerprint(HashAlg::Sha256).to_string();
  Ok((
    key
      .to_openssh()
      .map_err(|_| AppError::bad_request("SSH public key is invalid"))?,
    fingerprint,
  ))
}
pub(super) enum KeyIdentity {
  Personal(
    gity_entities::user_ssh_keys::Model,
    gity_identity::UserIdentity,
  ),
  Deploy(Vec<Value>),
}
pub(super) async fn authorized(
  state: &AppState,
  fingerprint: &str,
) -> Result<Option<KeyIdentity>, AppError> {
  if let Some(key) = state.identity.personal_ssh_key(fingerprint).await? {
    // Concurrent cross-mode registration is fail-closed, even if the deploy
    // creator/project is disabled and would not otherwise authenticate.
    if state
      .identity
      .deployment_fingerprint_exists(fingerprint)
      .await?
    {
      return Ok(None);
    }
    return Ok(
      state
        .identity
        .user_by_id(key.user_id)
        .await?
        .filter(|user| user.is_active())
        .map(|user| KeyIdentity::Personal(key, user)),
    );
  }
  let keys = state.repository_access.authorized_keys(fingerprint).await?;
  // Recheck the other table after the deploy lookup to catch a concurrent add.
  if keys.is_empty()
    || state
      .identity
      .personal_ssh_key(fingerprint)
      .await?
      .is_some()
  {
    return Ok(None);
  }
  Ok(Some(KeyIdentity::Deploy(
    keys
      .into_iter()
      .map(gity_persistence::model_value)
      .collect(),
  )))
}
pub(super) fn host_key(path: &Path) -> anyhow::Result<PrivateKey> {
  if path.exists() {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      anyhow::ensure!(
        std::fs::symlink_metadata(path)?.is_file()
          && std::fs::metadata(path)?.permissions().mode() & 0o077 == 0,
        "SSH host key must be a regular file accessible only by its owner"
      );
    }
    return Ok(russh::keys::load_secret_key(path, None)?);
  }
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  let key = PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519)?;
  let mut options = std::fs::OpenOptions::new();
  options.write(true).create_new(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  let mut file = options.open(path)?;
  file.write_all(key.to_openssh(LineEnding::LF)?.as_bytes())?;
  file.sync_all()?;
  Ok(key)
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
