//! Stored profiles for the CLI: one file, one active profile, 0600 permissions.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
  pub current: Option<String>,
  #[serde(default)]
  pub profiles: BTreeMap<String, Profile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
  pub host: String,
  pub token: String,
}

pub fn resolve_path(explicit: Option<PathBuf>) -> PathBuf {
  if let Some(path) = explicit {
    return path;
  }
  let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
  base.join("gity").join("config.toml")
}

pub fn load(path: &Path) -> Config {
  std::fs::read_to_string(path)
    .ok()
    .and_then(|text| toml::from_str(&text).ok())
    .unwrap_or_default()
}

pub fn save(path: &Path, config: &Config) -> Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)
      .with_context(|| format!("create config directory {}", parent.display()))?;
  }
  let text = toml::to_string_pretty(config).context("serialize configuration")?;
  std::fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
  restrict(path)?;
  Ok(())
}

/// Profiles hold bearer tokens, so keep the file readable by the owner only.
fn restrict(path: &Path) -> Result<()> {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, mode).with_context(|| format!("chmod {}", path.display()))?;
  }
  Ok(())
}

/// Host and token from flags, environment or the active profile, in that order.
pub fn resolve(options: &super::client::Options) -> Result<(String, String)> {
  let config = load(&options.config_file);
  let profile = config
    .current
    .as_ref()
    .and_then(|name| config.profiles.get(name));
  let host = options
    .host
    .clone()
    .or_else(|| profile.map(|p| p.host.clone()))
    .ok_or_else(|| {
      anyhow::anyhow!("no server configured: pass --host or run `gity auth login --host <url>`")
    })?;
  let token = options
    .token
    .clone()
    .or_else(|| profile.map(|p| p.token.clone()))
    .ok_or_else(|| anyhow::anyhow!("not signed in: pass --token or run `gity auth login`"))?;
  Ok((host.trim_end_matches('/').to_owned(), token))
}

pub fn store(options: &super::client::Options, host: &str, token: &str) -> Result<()> {
  let mut config = load(&options.config_file);
  let name = config
    .current
    .clone()
    .unwrap_or_else(|| "default".to_owned());
  config.profiles.insert(
    name.clone(),
    Profile {
      host: host.trim_end_matches('/').to_owned(),
      token: token.to_owned(),
    },
  );
  config.current = Some(name);
  save(&options.config_file, &config)
}
