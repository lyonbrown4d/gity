use crate::AppState;
use gity_core::AppError;
use russh::keys::{HashAlg, PrivateKey, PublicKey, ssh_key::LineEnding};
use serde_json::Value;
use std::{io::Write, path::Path};

pub(crate) fn canonical_key(raw: &str) -> Result<(String, String), AppError> {
  let mut key = PublicKey::from_openssh(raw)
    .map_err(|_| AppError::bad_request("deploy key public key is invalid"))?;
  key.set_comment("");
  let fingerprint = key.fingerprint(HashAlg::Sha256).to_string();
  Ok((
    key
      .to_openssh()
      .map_err(|_| AppError::bad_request("deploy key public key is invalid"))?,
    fingerprint,
  ))
}
pub(super) async fn authorized(
  state: &AppState,
  fingerprint: &str,
) -> Result<Vec<Value>, AppError> {
  state.db.all("SELECT k.* FROM project_deploy_keys k JOIN users u ON u.id=k.created_by_user_id JOIN projects p ON p.id=k.project_id WHERE k.fingerprint=? AND u.status='active' AND p.status='active' AND p.deleted_at IS NULL", vec![fingerprint.into()]).await
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
