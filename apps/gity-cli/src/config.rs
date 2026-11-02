//! glab-style configuration: one YAML file, a current host plus per-host tokens.
//!
//! ```yaml
//! host: http://127.0.0.1:8080
//! hosts:
//!   http://127.0.0.1:8080:
//!     token: gity_xxx
//!     user: admin
//! git_protocol: https
//! editor: null
//! ```
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
  pub host: Option<String>,
  #[serde(default)]
  pub hosts: BTreeMap<String, HostConfig>,
  #[serde(default = "default_protocol")]
  pub git_protocol: String,
  #[serde(default)]
  pub editor: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HostConfig {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub token: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,
}

fn default_protocol() -> String {
  "https".to_owned()
}

#[derive(Clone, Debug)]
pub struct Resolved {
  pub host: String,
  pub token: String,
}

pub fn resolve_path(explicit: Option<PathBuf>) -> PathBuf {
  if let Some(path) = explicit {
    return path;
  }
  let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
  base.join("gity-cli").join("config.yaml")
}

pub fn load(path: &Path) -> Config {
  std::fs::read_to_string(path)
    .ok()
    .and_then(|text| serde_yaml_ng::from_str(&text).ok())
    .unwrap_or_default()
}

pub fn save(path: &Path, config: &Config) -> Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)
      .with_context(|| format!("create config directory {}", parent.display()))?;
  }
  let text = serde_yaml_ng::to_string(config).context("serialize configuration")?;
  std::fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
  restrict(path)?;
  Ok(())
}

/// The file stores bearer tokens, so only the owner may read it.
fn restrict(path: &Path) -> Result<()> {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, mode).with_context(|| format!("chmod {}", path.display()))?;
  }
  Ok(())
}

fn normalise(host: &str) -> String {
  host.trim_end_matches('/').to_owned()
}

/// Flags and environment win over the file, matching `glab` and `gh`.
pub fn resolve(options: &super::client::Options) -> Result<Resolved> {
  let config = load(&options.config_file);
  let host = options
    .host
    .clone()
    .or_else(|| config.host.clone())
    .ok_or_else(|| {
      anyhow::anyhow!("no server configured: pass --host or run `gity auth login --host <url>`")
    })?;
  let host = normalise(&host);
  let token = options
    .token
    .clone()
    .or_else(|| {
      config
        .hosts
        .get(&host)
        .and_then(|entry| entry.token.clone())
    })
    .ok_or_else(|| {
      anyhow::anyhow!("not signed in to {host}: pass --token or run `gity auth login`")
    })?;
  Ok(Resolved { host, token })
}

/// Records the token for `host` and makes that host current.
pub fn store_host(
  options: &super::client::Options,
  host: &str,
  token: &str,
  user: Option<&str>,
) -> Result<()> {
  let mut config = load(&options.config_file);
  let host = normalise(host);
  config.hosts.insert(
    host.clone(),
    HostConfig {
      token: Some(token.to_owned()),
      user: user.map(str::to_owned),
    },
  );
  config.host = Some(host);
  save(&options.config_file, &config)
}

pub fn remove_host(options: &super::client::Options, host: &str) -> Result<bool> {
  let mut config = load(&options.config_file);
  let host = normalise(host);
  let removed = config.hosts.remove(&host).is_some();
  if config.host.as_deref() == Some(host.as_str()) {
    config.host = config.hosts.keys().next().cloned();
  }
  save(&options.config_file, &config)?;
  Ok(removed)
}

/// `gity config get|set` works on the small set of top-level keys.
pub fn get_key(config: &Config, key: &str) -> Result<String> {
  let value = match key {
    "host" => config.host.clone().unwrap_or_default(),
    "git_protocol" => config.git_protocol.clone(),
    "editor" => config.editor.clone().unwrap_or_default(),
    other => bail!("unknown key '{other}'; try host, git_protocol or editor"),
  };
  Ok(value)
}

pub fn set_key(config: &mut Config, key: &str, value: &str) -> Result<()> {
  match key {
    "host" => config.host = Some(normalise(value)),
    "git_protocol" => {
      if !matches!(value, "https" | "ssh") {
        bail!("git_protocol must be https or ssh");
      }
      config.git_protocol = value.to_owned();
    }
    "editor" => config.editor = Some(value.to_owned()),
    other => bail!("unknown key '{other}'; try host, git_protocol or editor"),
  }
  Ok(())
}
