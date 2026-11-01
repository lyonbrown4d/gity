use config::{Config, ConfigBuilder, Environment, File};
use gity_gateway::GatewaySettings;
use serde::Deserialize;
use std::{net::SocketAddr, path::Path};

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayFile {
  #[serde(default = "default_observability")]
  pub observability: gity_observability::ObservabilityConfig,
  #[serde(default = "default_http_listen")]
  http_listen: SocketAddr,
  #[serde(default = "default_ssh_listen")]
  ssh_listen: SocketAddr,
  #[serde(default = "default_main_http_origin")]
  main_http_origin: String,
  package_http_origin: Option<String>,
  #[serde(default = "default_public_scheme")]
  public_scheme: String,
  #[serde(default = "default_main_ssh_address")]
  main_ssh_address: String,
  #[serde(default)]
  allow_insecure_private_network: bool,
  #[serde(default = "default_request_timeout")]
  request_timeout_seconds: u64,
  #[serde(default = "default_ssh_connect_timeout")]
  ssh_connect_timeout_seconds: u64,
  #[serde(default = "default_ssh_session_timeout")]
  ssh_session_timeout_seconds: u64,
  #[serde(default = "default_shutdown_timeout")]
  shutdown_timeout_seconds: u64,
  #[serde(default = "default_http_limit")]
  max_http_requests: usize,
  #[serde(default = "default_ssh_limit")]
  max_ssh_sessions: usize,
}

#[derive(Default)]
pub struct GatewayOverrides {
  pub http_listen: Option<SocketAddr>,
  pub ssh_listen: Option<SocketAddr>,
  pub main_http_origin: Option<String>,
  pub package_http_origin: Option<String>,
  pub public_scheme: Option<String>,
  pub main_ssh_address: Option<String>,
  pub allow_insecure_private_network: Option<bool>,
  pub request_timeout_seconds: Option<u64>,
  pub ssh_connect_timeout_seconds: Option<u64>,
  pub ssh_session_timeout_seconds: Option<u64>,
  pub shutdown_timeout_seconds: Option<u64>,
  pub max_http_requests: Option<usize>,
  pub max_ssh_sessions: Option<usize>,
}

impl GatewayFile {
  pub fn load(path: Option<&Path>, overrides: GatewayOverrides) -> anyhow::Result<Self> {
    let mut builder = Config::builder()
      .set_default("http_listen", default_http_listen().to_string())?
      .set_default("ssh_listen", default_ssh_listen().to_string())?
      .set_default("main_http_origin", default_main_http_origin())?
      .set_default("main_ssh_address", default_main_ssh_address())?
      .set_default("allow_insecure_private_network", false)?
      .set_default("request_timeout_seconds", default_request_timeout())?
      .set_default("ssh_connect_timeout_seconds", default_ssh_connect_timeout())?
      .set_default("ssh_session_timeout_seconds", default_ssh_session_timeout())?
      .set_default("shutdown_timeout_seconds", default_shutdown_timeout())?
      .set_default("max_http_requests", default_http_limit() as i64)?
      .set_default("max_ssh_sessions", default_ssh_limit() as i64)?;
    if let Some(path) = path {
      builder = builder.add_source(File::from(path).required(true));
    } else {
      builder = builder.add_source(File::with_name("config/gateway").required(false));
    }
    builder = builder.add_source(
      Environment::with_prefix("GITY_GATEWAY")
        .prefix_separator("_")
        .separator("__")
        .try_parsing(true),
    );
    builder = set_override(
      builder,
      "http_listen",
      overrides.http_listen.map(|value| value.to_string()),
    )?;
    builder = set_override(
      builder,
      "ssh_listen",
      overrides.ssh_listen.map(|value| value.to_string()),
    )?;
    builder = set_override(builder, "main_http_origin", overrides.main_http_origin)?;
    builder = set_override(builder, "public_scheme", overrides.public_scheme)?;
    builder = set_override(
      builder,
      "package_http_origin",
      overrides.package_http_origin,
    )?;
    builder = set_override(builder, "main_ssh_address", overrides.main_ssh_address)?;
    builder = set_override(
      builder,
      "allow_insecure_private_network",
      overrides.allow_insecure_private_network,
    )?;
    builder = set_override(
      builder,
      "request_timeout_seconds",
      overrides.request_timeout_seconds,
    )?;
    builder = set_override(
      builder,
      "ssh_connect_timeout_seconds",
      overrides.ssh_connect_timeout_seconds,
    )?;
    builder = set_override(
      builder,
      "ssh_session_timeout_seconds",
      overrides.ssh_session_timeout_seconds,
    )?;
    builder = set_override(
      builder,
      "shutdown_timeout_seconds",
      overrides.shutdown_timeout_seconds,
    )?;
    builder = set_usize_override(builder, "max_http_requests", overrides.max_http_requests)?;
    builder = set_usize_override(builder, "max_ssh_sessions", overrides.max_ssh_sessions)?;
    builder
      .build()
      .map_err(|_| anyhow::anyhow!("gateway configuration is invalid"))?
      .try_deserialize()
      .map_err(|_| anyhow::anyhow!("gateway configuration is invalid"))
  }

  pub fn into_runtime(self) -> anyhow::Result<GatewaySettings> {
    let settings = GatewaySettings {
      http_listen: self.http_listen,
      ssh_listen: self.ssh_listen,
      main_http_origin: self.main_http_origin,
      package_http_origin: self.package_http_origin,
      public_scheme: self.public_scheme,
      main_ssh_address: self.main_ssh_address,
      allow_insecure_private_network: self.allow_insecure_private_network,
      request_timeout_seconds: self.request_timeout_seconds,
      ssh_connect_timeout_seconds: self.ssh_connect_timeout_seconds,
      ssh_session_timeout_seconds: self.ssh_session_timeout_seconds,
      shutdown_timeout_seconds: self.shutdown_timeout_seconds,
      max_http_requests: self.max_http_requests,
      max_ssh_sessions: self.max_ssh_sessions,
    };
    settings.validate().map_err(anyhow::Error::msg)?;
    Ok(settings)
  }
}

fn set_override<T: Into<config::Value>>(
  builder: ConfigBuilder<config::builder::DefaultState>,
  key: &str,
  value: Option<T>,
) -> anyhow::Result<ConfigBuilder<config::builder::DefaultState>> {
  match value {
    Some(value) => builder
      .set_override(key, value)
      .map_err(|_| anyhow::anyhow!("invalid gateway command-line override")),
    None => Ok(builder),
  }
}

fn set_usize_override(
  builder: ConfigBuilder<config::builder::DefaultState>,
  key: &str,
  value: Option<usize>,
) -> anyhow::Result<ConfigBuilder<config::builder::DefaultState>> {
  let Some(value) = value else {
    return Ok(builder);
  };
  let value =
    i64::try_from(value).map_err(|_| anyhow::anyhow!("gateway command-line limit is too large"))?;
  builder
    .set_override(key, value)
    .map_err(|_| anyhow::anyhow!("invalid gateway command-line override"))
}

fn default_http_listen() -> SocketAddr {
  "0.0.0.0:8080".parse().unwrap()
}
fn default_ssh_listen() -> SocketAddr {
  "0.0.0.0:2222".parse().unwrap()
}
fn default_main_http_origin() -> String {
  "https://gity-server".into()
}
fn default_main_ssh_address() -> String {
  "gity-server:2222".into()
}
fn default_request_timeout() -> u64 {
  300
}
fn default_ssh_connect_timeout() -> u64 {
  10
}
fn default_ssh_session_timeout() -> u64 {
  86_400
}
fn default_shutdown_timeout() -> u64 {
  10
}
fn default_http_limit() -> usize {
  1024
}
fn default_ssh_limit() -> usize {
  256
}

fn default_public_scheme() -> String {
  "http".into()
}

fn default_observability() -> gity_observability::ObservabilityConfig {
  gity_observability::ObservabilityConfig::for_service("gity-gateway")
}
