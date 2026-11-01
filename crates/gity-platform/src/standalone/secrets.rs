#[cfg(feature = "oci")]
use gity_oci::{MirrorCredential, MirrorCredentialProvider, MirrorResolveError};
#[cfg(feature = "oci")]
use serde::Deserialize;

#[cfg(feature = "oci")]
pub struct ConfiguredSecrets;
#[cfg(feature = "oci")]
#[derive(Deserialize)]
struct BasicSecret {
  username: String,
  password: String,
}
#[cfg(feature = "oci")]
#[async_trait::async_trait]
impl MirrorCredentialProvider for ConfiguredSecrets {
  async fn resolve_basic(
    &self,
    reference: &str,
  ) -> Result<Option<MirrorCredential>, MirrorResolveError> {
    let raw = if let Some(name) = reference.strip_prefix("env:") {
      std::env::var(name).map_err(|_| MirrorResolveError::Unavailable)?
    } else if let Some(path) = reference.strip_prefix("file:") {
      let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| MirrorResolveError::Unavailable)?;
      if bytes.len() > 65536 {
        return Err(MirrorResolveError::InvalidResponse);
      }
      String::from_utf8(bytes).map_err(|_| MirrorResolveError::InvalidResponse)?
    } else {
      return Err(MirrorResolveError::InvalidResponse);
    };
    let value: BasicSecret =
      serde_json::from_str(&raw).map_err(|_| MirrorResolveError::InvalidResponse)?;
    Ok(Some(MirrorCredential {
      username: value.username,
      password: value.password,
    }))
  }
}
pub(super) async fn write_private(path: &std::path::Path, bytes: &[u8]) -> anyhow::Result<()> {
  use tokio::io::AsyncWriteExt;
  let parent = path
    .parent()
    .ok_or_else(|| anyhow::anyhow!("private runtime directory required"))?;
  tokio::fs::create_dir_all(parent).await?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).await?;
  }
  if let Ok(metadata) = tokio::fs::symlink_metadata(path).await {
    anyhow::ensure!(
      !metadata.file_type().is_symlink(),
      "private runtime path must not be a symlink"
    );
  }
  let mut options = tokio::fs::OpenOptions::new();
  options.create(true).truncate(true).write(true);
  #[cfg(unix)]
  options.mode(0o600);
  let mut file = options.open(path).await?;
  file.write_all(bytes).await?;
  file.sync_all().await?;
  Ok(())
}
