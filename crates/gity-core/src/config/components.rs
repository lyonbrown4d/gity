use serde::Deserialize;
use std::{
  collections::{BTreeMap, BTreeSet},
  net::SocketAddr,
  path::PathBuf,
};

#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ComponentConfig {
  pub authority: AuthorityConfig,
  pub registry: RegistryConfig,
  pub runner: RunnerConfig,
  pub standalone: bool,
}
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuthorityConfig {
  pub enabled: bool,
  pub address: SocketAddr,
  pub token: String,
  pub tls_certificate: Option<PathBuf>,
  pub tls_key: Option<PathBuf>,
  pub client_ca: Option<PathBuf>,
}
impl Default for AuthorityConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      address: "127.0.0.1:9100".parse().unwrap(),
      token: String::new(),
      tls_certificate: None,
      tls_key: None,
      client_ca: None,
    }
  }
}
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RegistryConfig {
  pub enabled: bool,
  pub address: SocketAddr,
  pub engine_url: String,
  pub engine_binary: PathBuf,
  pub engine_configs: Vec<PathBuf>,
  pub mirrors: BTreeMap<String, MirrorUpstreamConfig>,
  pub allowed_upstream_authorities: BTreeSet<String>,
}
impl Default for RegistryConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      address: "0.0.0.0:5000".parse().unwrap(),
      engine_url: "http://127.0.0.1:18000".into(),
      engine_binary: "/usr/local/bin/angos".into(),
      engine_configs: Vec::new(),
      mirrors: BTreeMap::new(),
      allowed_upstream_authorities: BTreeSet::new(),
    }
  }
}
#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MirrorUpstreamConfig {
  pub host: String,
  pub credential_secret_ref: Option<String>,
  pub allow_all_projects: bool,
  pub allowed_projects: BTreeSet<i64>,
}
impl MirrorUpstreamConfig {
  pub fn allows_project(&self, project: i64) -> bool {
    self.allow_all_projects || self.allowed_projects.contains(&project)
  }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RunnerConfig {
  pub enabled: bool,
  pub executable: PathBuf,
  pub token_directory: PathBuf,
  pub work_root: PathBuf,
  pub container_bin: PathBuf,
}
impl Default for RunnerConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      executable: "/usr/local/bin/gity-runner".into(),
      token_directory: "data/runner-tokens".into(),
      work_root: "data/runner".into(),
      container_bin: "docker".into(),
    }
  }
}

impl Default for super::StorageConfig {
  fn default() -> Self {
    Self {
      driver: "local".into(),
      root: "data/storage".into(),
      s3_bucket: String::new(),
      s3_region: "us-east-1".into(),
      s3_endpoint: String::new(),
      s3_access_key: String::new(),
      s3_secret_key: String::new(),
      s3_use_path_style: true,
    }
  }
}
