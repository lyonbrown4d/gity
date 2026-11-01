use crate::AppError;
mod cluster;
mod components;
mod ssh;
pub use cluster::{GitClusterConfig, GitStorageNodeConfig};
pub use components::{AuthorityConfig, ComponentConfig, MirrorUpstreamConfig, RegistryConfig};
use serde::Deserialize;
pub use ssh::SshConfig;
use std::path::{Path, PathBuf};
#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Settings {
  pub git_cluster: Option<GitClusterConfig>,
  pub components: ComponentConfig,
  pub address: String,
  pub base_url: String,
  pub database_url: String,
  pub database_node_id: u16,
  pub repo_root: PathBuf,
  pub git_bin: PathBuf,
  pub git_hook: Option<PathBuf>,
  pub storage_root: PathBuf,
  pub storage: StorageConfig,
  pub web_root: PathBuf,
  pub observability: ObservabilityConfig,
  pub search: SearchConfig,
  pub ssh: SshConfig,
}
#[derive(Clone, Deserialize)]
pub struct SearchConfig {
  pub index_enabled: bool,
  pub index_root: PathBuf,
  pub refresh_interval_seconds: u64,
  pub max_files_per_project: usize,
  pub max_file_size_bytes: usize,
}
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
  pub driver: String,
  pub root: PathBuf,
  #[serde(default)]
  pub s3_bucket: String,
  pub s3_region: String,
  #[serde(default)]
  pub s3_endpoint: String,
  #[serde(default)]
  pub s3_access_key: String,
  #[serde(default)]
  pub s3_secret_key: String,
  pub s3_use_path_style: bool,
}
pub use gity_observability::ObservabilityConfig;
#[derive(Deserialize)]
struct Raw {
  #[serde(default)]
  components: ComponentConfig,
  http: Http,
  database: Database,
  git: Git,
  storage: StorageConfig,
  web: Web,
  observability: ObservabilityConfig,
  search: SearchConfig,
  ssh: SshConfig,
}
#[derive(Deserialize)]
struct Http {
  address: String,
  base_url: String,
}
#[derive(Deserialize)]
struct Database {
  dsn: String,
  driver: String,
  node_id: u16,
}
#[derive(Deserialize)]
struct Git {
  #[serde(default)]
  cluster: Option<GitClusterConfig>,
  repo_root: PathBuf,
  bin: PathBuf,
  #[serde(default)]
  hook: Option<PathBuf>,
}
#[derive(Deserialize)]
struct Web {
  root: PathBuf,
}

impl Settings {
  pub fn load() -> Self {
    Self::try_load(None).expect("valid Gity configuration")
  }
  /// Defaults < optional config/gity.toml < explicit configuration file < GITY_*.
  pub fn try_load(path: Option<&Path>) -> Result<Self, AppError> {
    dotenvy::dotenv().ok();
    let mut builder = config::Config::builder();
    for (key, value) in [
      ("http.address", "0.0.0.0:8080"),
      ("http.base_url", "http://localhost:8080"),
      ("database.dsn", "sqlite://data/gity.db?mode=rwc"),
      ("database.driver", "sqlite"),
      ("git.repo_root", "data/repos"),
      ("git.bin", "git"),
      ("ssh.address", "0.0.0.0:2222"),
      ("ssh.host_key", "data/ssh_host_ed25519"),
      ("storage.driver", "local"),
      ("storage.root", "data/storage"),
      ("storage.s3_region", "us-east-1"),
      ("web.root", "dist/web"),
      ("search.index_root", "data/search-index"),
      ("observability.service_name", "gity-server"),
      ("observability.log_format", "json"),
      ("observability.log_level", "gity=info,tower_http=info"),
      ("observability.metrics_address", "127.0.0.1:9090"),
    ] {
      builder = builder.set_default(key, value).map_err(config_error)?;
    }
    builder = builder
      .set_default("ssh.enabled", false)
      .map_err(config_error)?
      .set_default("ssh.max_connections", 64i64)
      .map_err(config_error)?
      .set_default("ssh.operation_timeout_seconds", 900i64)
      .map_err(config_error)?
      .set_default("search.index_enabled", true)
      .map_err(config_error)?
      .set_default("search.refresh_interval_seconds", 300i64)
      .map_err(config_error)?
      .set_default("search.max_files_per_project", 5000i64)
      .map_err(config_error)?
      .set_default("search.max_file_size_bytes", 524288i64)
      .map_err(config_error)?
      .set_default("database.node_id", 1)
      .map_err(config_error)?
      .set_default("storage.s3_use_path_style", true)
      .map_err(config_error)?
      .set_default("observability.metrics_enabled", true)
      .map_err(config_error)?
      .add_source(config::File::with_name("config/gity").required(false));
    let environment_path = std::env::var_os("GITY_CONFIG").map(PathBuf::from);
    if let Some(path) = path.or(environment_path.as_deref()) {
      builder = builder.add_source(config::File::from(path).required(true));
    }
    let mut raw: Raw = builder
      .add_source(
        config::Environment::with_prefix("GITY")
          .prefix_separator("_")
          .separator("__")
          .try_parsing(true),
      )
      .build()
      .map_err(config_error)?
      .try_deserialize()
      .map_err(config_error)?;
    raw.ssh.validate()?;
    if let Some(cluster) = &mut raw.git.cluster {
      cluster.load_credentials()?;
      cluster.validate()?;
    }
    if raw.database.node_id >= 1024 {
      return Err(AppError::bad_request("database.node_id must be 0–1023"));
    }
    if raw.search.refresh_interval_seconds == 0
      || raw.search.max_files_per_project == 0
      || raw.search.max_files_per_project > 100_000
      || raw.search.max_file_size_bytes == 0
      || raw.search.max_file_size_bytes > 10 * 1024 * 1024
    {
      return Err(AppError::bad_request(
        "invalid search refresh interval or index limits",
      ));
    }
    if !["sqlite", "postgres", "postgresql", "mysql"].contains(&raw.database.driver.as_str()) {
      return Err(AppError::bad_request("unsupported database driver"));
    }
    if !["json", "text"].contains(&raw.observability.log_format.as_str()) {
      return Err(AppError::bad_request(
        "observability.log_format must be json or text",
      ));
    }
    let mut address = raw.http.address;
    if address.starts_with(':') {
      address.insert_str(0, "0.0.0.0");
    }
    address
      .parse::<std::net::SocketAddr>()
      .map_err(|_| AppError::bad_request("http.address must be an IP address and port"))?;
    let database_url = if let Some(path) = raw.database.dsn.strip_prefix("file:") {
      format!(
        "sqlite://{}?mode=rwc",
        path.split('?').next().unwrap_or("data/gity.db")
      )
    } else {
      raw.database.dsn
    };
    let matching_scheme = match raw.database.driver.as_str() {
      "sqlite" => database_url.starts_with("sqlite:"),
      "postgres" | "postgresql" => {
        database_url.starts_with("postgres:") || database_url.starts_with("postgresql:")
      }
      "mysql" => database_url.starts_with("mysql:"),
      _ => false,
    };
    if !matching_scheme {
      return Err(AppError::bad_request(
        "database.driver and DSN scheme do not match",
      ));
    }
    Ok(Self {
      git_cluster: raw.git.cluster,
      components: raw.components,
      address,
      base_url: raw.http.base_url,
      database_url,
      database_node_id: raw.database.node_id,
      repo_root: raw.git.repo_root,
      git_bin: raw.git.bin,
      git_hook: raw.git.hook,
      storage_root: raw.storage.root.clone(),
      storage: raw.storage,
      web_root: raw.web.root,
      observability: raw.observability,
      search: raw.search,
      ssh: raw.ssh,
    })
  }
}
fn config_error(_error: config::ConfigError) -> AppError {
  // Parser errors can embed the rejected value, including a credential.
  tracing::error!("configuration validation failed; check field types and required settings");
  AppError::bad_request(
    "invalid configuration; check configuration field types and required settings",
  )
}
