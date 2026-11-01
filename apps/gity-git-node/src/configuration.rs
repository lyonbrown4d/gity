use config::{Config, Environment, File};
use gity_git_node::GitNodeSettings;
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Deserialize)]
#[serde(default)]
struct NodeConfig {
  observability: gity_observability::ObservabilityConfig,
  id: String,
  address: SocketAddr,
  repo_root: PathBuf,
  git_bin: PathBuf,
  hook_executable: PathBuf,
  token: String,
  read_only: bool,
  replica_max_bytes: u64,
  replica_timeout_secs: u64,
  #[serde(alias = "tls_cert")]
  tls_certificate: Option<PathBuf>,
  #[serde(alias = "tls_key")]
  tls_key: Option<PathBuf>,
  client_ca: Option<PathBuf>,
}

impl Default for NodeConfig {
  fn default() -> Self {
    Self {
      observability: gity_observability::ObservabilityConfig::for_service("gity-git-node"),
      id: "git-1".into(),
      address: "127.0.0.1:9101".parse().expect("static socket address"),
      repo_root: "data/repos".into(),
      git_bin: "git".into(),
      hook_executable: "/usr/local/bin/gity-git-hook".into(),
      token: String::new(),
      read_only: false,
      replica_max_bytes: 10_737_418_240,
      replica_timeout_secs: 3600,
      tls_certificate: None,
      tls_key: None,
      client_ca: None,
    }
  }
}

#[derive(Default)]
pub struct CliOverrides {
  pub id: Option<String>,
  pub address: Option<SocketAddr>,
  pub repo_root: Option<PathBuf>,
  pub git_bin: Option<PathBuf>,
  pub hook_executable: Option<PathBuf>,
  pub token: Option<String>,
  pub read_only: Option<bool>,
  pub replica_max_bytes: Option<u64>,
  pub replica_timeout_secs: Option<u64>,
  pub tls_certificate: Option<PathBuf>,
  pub tls_key: Option<PathBuf>,
  pub client_ca: Option<PathBuf>,
}

pub fn load_settings(
  config_path: Option<PathBuf>,
  cli: CliOverrides,
) -> anyhow::Result<GitNodeSettings> {
  load_runtime(config_path, cli).map(|(settings, _)| settings)
}
pub fn load_runtime(
  config_path: Option<PathBuf>,
  cli: CliOverrides,
) -> anyhow::Result<(GitNodeSettings, gity_observability::ObservabilityConfig)> {
  let mut builder = Config::builder();
  if let Some(path) = config_path {
    builder = builder.add_source(File::from(path).required(true));
  }
  let mut settings: NodeConfig = builder
    .add_source(
      Environment::with_prefix("GITY_GIT_NODE")
        .prefix_separator("_")
        .separator("__")
        .try_parsing(true),
    )
    .build()
    .and_then(|value| value.try_deserialize())
    .map_err(|_| {
      anyhow::anyhow!("invalid Git node configuration; check field types and required settings")
    })?;

  replace(&mut settings.id, cli.id);
  replace(&mut settings.address, cli.address);
  replace(&mut settings.repo_root, cli.repo_root);
  replace(&mut settings.git_bin, cli.git_bin);
  replace(&mut settings.hook_executable, cli.hook_executable);
  replace(&mut settings.token, cli.token);
  replace(&mut settings.read_only, cli.read_only);
  replace(&mut settings.replica_max_bytes, cli.replica_max_bytes);
  replace(&mut settings.replica_timeout_secs, cli.replica_timeout_secs);
  replace_optional(&mut settings.tls_certificate, cli.tls_certificate);
  replace_optional(&mut settings.tls_key, cli.tls_key);
  replace_optional(&mut settings.client_ca, cli.client_ca);

  Ok((
    GitNodeSettings {
      id: settings.id,
      address: settings.address,
      repo_root: settings.repo_root,
      git_bin: settings.git_bin,
      hook_executable: settings.hook_executable,
      token: settings.token,
      read_only: settings.read_only,
      replica_max_bytes: settings.replica_max_bytes,
      replica_timeout_secs: settings.replica_timeout_secs,
      tls_certificate: settings.tls_certificate,
      tls_key: settings.tls_key,
      client_ca: settings.client_ca,
    },
    settings.observability,
  ))
}

fn replace<T>(value: &mut T, override_value: Option<T>) {
  if let Some(override_value) = override_value {
    *value = override_value;
  }
}

fn replace_optional<T>(value: &mut Option<T>, override_value: Option<T>) {
  if let Some(override_value) = override_value {
    *value = Some(override_value);
  }
}
