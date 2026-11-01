use crate::credentials::EnvironmentCredentials;
use anyhow::anyhow;
use config::{Config, Environment, File};
use gity_git_wire::NodeEndpoint;
use gity_oci::{MirrorCredentialProvider, MirrorSettings, OciSettings, UpstreamPolicy};
use gity_runtime::AngosConfig;
use serde::Deserialize;
use std::{
  collections::BTreeMap,
  net::SocketAddr,
  path::{Path, PathBuf},
  sync::Arc,
  time::Duration,
};

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrySettings {
  #[serde(default = "default_observability")]
  observability: gity_observability::ObservabilityConfig,
  #[serde(default)]
  packages: crate::package_settings::PackageSettings,
  address: SocketAddr,
  #[serde(default = "default_timeout")]
  request_timeout_seconds: u64,
  angos: AngosFile,
  authority: AuthorityFile,
  oci: OciFile,
  #[serde(default)]
  mirror_secrets: BTreeMap<String, CredentialEnvironment>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AngosFile {
  executable: PathBuf,
  #[serde(default)]
  config_files: Vec<PathBuf>,
  health_url: String,
  #[serde(default = "default_start_timeout")]
  startup_timeout_seconds: u64,
  #[serde(default = "default_restart_limit")]
  restart_limit: u32,
  #[serde(default = "default_stop_timeout")]
  shutdown_timeout_seconds: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityFile {
  #[serde(default)]
  peer_token: String,
  token_file: Option<PathBuf>,
  id: String,
  uri: String,
  ca_certificate: Option<PathBuf>,
  client_certificate: Option<PathBuf>,
  client_key: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OciFile {
  upstream: String,
  #[serde(default)]
  upstream_policy: UpstreamPolicy,
  #[serde(default)]
  mirror: Option<MirrorSettings>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CredentialEnvironment {
  pub(crate) username: String,
  pub(crate) password: String,
}

pub struct RegistryRuntimeConfig {
  pub observability: gity_observability::ObservabilityConfig,
  pub packages: crate::package_settings::PackageSettings,
  pub address: SocketAddr,
  pub angos: AngosConfig,
  pub authority_endpoint: NodeEndpoint,
  pub oci: OciSettings,
  pub credentials: Arc<dyn MirrorCredentialProvider>,
}

#[derive(Default)]
pub struct RegistryOverrides {
  pub address: Option<SocketAddr>,
  pub request_timeout_seconds: Option<u64>,
  pub angos_executable: Option<PathBuf>,
  pub authority_uri: Option<String>,
  pub oci_upstream: Option<String>,
}

impl RegistrySettings {
  pub fn load(path: Option<&Path>, overrides: RegistryOverrides) -> anyhow::Result<Self> {
    let mut builder = Config::builder()
      .set_default("address", "0.0.0.0:8081")?
      .set_default("request_timeout_seconds", 30)?
      .set_default("angos.executable", "/usr/local/bin/angos")?
      .set_default("angos.config_files", Vec::<String>::new())?
      .set_default("angos.health_url", "http://127.0.0.1:8000/readyz")?
      .set_default("angos.startup_timeout_seconds", 30)?
      .set_default("angos.restart_limit", 5)?
      .set_default("angos.shutdown_timeout_seconds", 10)?
      .set_default("authority.id", "main")?
      .set_default("authority.uri", "http://127.0.0.1:9102")?
      .set_default("oci.upstream", "http://127.0.0.1:8000")?;
    if let Some(path) = path {
      builder = builder.add_source(File::from(path).required(true));
    } else {
      builder = builder.add_source(File::with_name("config/registry").required(false));
    }
    builder = builder.add_source(
      Environment::with_prefix("GITY_REGISTRY")
        .prefix_separator("_")
        .separator("__")
        .try_parsing(true),
    );
    if let Some(address) = overrides.address {
      builder = builder.set_override("address", address.to_string())?;
    }
    if let Some(timeout) = overrides.request_timeout_seconds {
      builder = builder.set_override("request_timeout_seconds", timeout)?;
    }
    if let Some(path) = overrides.angos_executable {
      builder = builder.set_override("angos.executable", path.to_string_lossy().to_string())?;
    }
    if let Some(uri) = overrides.authority_uri {
      builder = builder.set_override("authority.uri", uri)?;
    }
    if let Some(upstream) = overrides.oci_upstream {
      builder = builder.set_override("oci.upstream", upstream)?;
    }
    let raw: RegistrySettings = builder
      .build()
      .map_err(|_| anyhow!("registry configuration is invalid"))?
      .try_deserialize()
      .map_err(|_| anyhow!("registry configuration is invalid"))?;
    Ok(raw)
  }

  pub async fn into_runtime(
    self,
    authority_peer_token: String,
  ) -> anyhow::Result<RegistryRuntimeConfig> {
    let request_timeout = Duration::from_secs(self.request_timeout_seconds);
    if request_timeout.is_zero() {
      return Err(anyhow!("registry request timeout must be positive"));
    }
    let angos = AngosConfig {
      executable: self.angos.executable,
      config_files: self.angos.config_files,
      health_url: self.angos.health_url,
      startup_timeout: Duration::from_secs(self.angos.startup_timeout_seconds),
      restart_limit: self.angos.restart_limit,
      shutdown_timeout: Duration::from_secs(self.angos.shutdown_timeout_seconds),
      environment: BTreeMap::new(),
    };
    angos
      .validate()
      .map_err(|_| anyhow!("Angos configuration is invalid"))?;
    let authority_endpoint = NodeEndpoint {
      id: self.authority.id,
      uri: self.authority.uri,
      token: if !authority_peer_token.is_empty() {
        authority_peer_token
      } else if let Some(path) = self.authority.token_file {
        let metadata = tokio::fs::metadata(&path)
          .await
          .map_err(|_| anyhow!("cannot read authority credential file"))?;
        if !metadata.is_file() || metadata.len() > 4096 {
          return Err(anyhow!("invalid authority credential file"));
        }
        tokio::fs::read_to_string(path)
          .await
          .map_err(|_| anyhow!("cannot read authority credential file"))?
          .trim_end_matches(['\r', '\n'])
          .to_owned()
      } else {
        self.authority.peer_token
      },
      ca_certificate: read_pem(self.authority.ca_certificate).await?,
      client_certificate: read_pem(self.authority.client_certificate).await?,
      client_key: read_pem(self.authority.client_key).await?,
    };
    if authority_endpoint.token.len() < 32 {
      return Err(anyhow!("authority peer token must be at least 32 bytes"));
    }
    let oci = OciSettings {
      upstream: self.oci.upstream,
      request_timeout,
      upstream_policy: self.oci.upstream_policy,
      mirror: self.oci.mirror,
    };
    oci
      .validate()
      .map_err(|_| anyhow!("OCI configuration is invalid"))?;
    let credentials = Arc::new(EnvironmentCredentials(self.mirror_secrets));
    Ok(RegistryRuntimeConfig {
      observability: self.observability,
      packages: self.packages,
      address: self.address,
      angos,
      authority_endpoint,
      oci,
      credentials,
    })
  }
}

async fn read_pem(path: Option<PathBuf>) -> anyhow::Result<Option<String>> {
  match path {
    Some(path) => tokio::fs::read_to_string(path)
      .await
      .map(Some)
      .map_err(|_| anyhow!("cannot read authority TLS file")),
    None => Ok(None),
  }
}

fn default_timeout() -> u64 {
  30
}
fn default_start_timeout() -> u64 {
  30
}
fn default_restart_limit() -> u32 {
  5
}
fn default_stop_timeout() -> u64 {
  10
}

fn default_observability() -> gity_observability::ObservabilityConfig {
  gity_observability::ObservabilityConfig::for_service("gity-registry")
}
