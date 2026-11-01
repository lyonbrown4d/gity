use serde::Deserialize;
use std::{
  collections::{BTreeMap, BTreeSet},
  time::Duration,
};

#[derive(Clone, Debug)]
pub struct OciSettings {
  /// Private Angos endpoint. Binding it to loopback or a private network is
  /// the deployment's responsibility and is documented in deploy/oci.
  pub upstream: String,
  pub request_timeout: Duration,
  pub upstream_policy: UpstreamPolicy,
  pub mirror: Option<MirrorSettings>,
}

/// Gity-side controls for Angos's read-through mirror mode. Secret values stay
/// in the private engine's secret store; only references cross this boundary.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MirrorSettings {
  /// Stable Gity alias to an allowlisted upstream registry host.
  pub registries: BTreeMap<String, MirrorRegistry>,
  /// Exact HTTPS hosts allowed for token realms and redirects. The registry
  /// host itself must also appear here.
  pub allowed_hosts: BTreeSet<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MirrorRegistry {
  pub host: String,
  pub credential_secret_ref: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamPolicy {
  /// Exact allowed hosts for token service realms. Registry response
  /// redirects are constrained to the configured Angos origin.
  pub allowed_hosts: BTreeSet<String>,
}

impl OciSettings {
  pub fn validate(&self) -> Result<(), String> {
    let url = reqwest::Url::parse(&self.upstream).map_err(|error| error.to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
      return Err("OCI upstream must be an absolute HTTP(S) URL".into());
    }
    if self.request_timeout.is_zero() {
      return Err("OCI request timeout must be positive".into());
    }
    if let Some(mirror) = &self.mirror {
      if mirror.registries.is_empty() {
        return Err("OCI mirror requires an upstream allowlist".into());
      }
      if mirror.allowed_hosts.is_empty() {
        return Err("OCI mirror requires an outbound host allowlist".into());
      }
      for (alias, registry) in &mirror.registries {
        if alias.is_empty() || alias.contains('/') || alias.contains('@') {
          return Err(format!("invalid mirror registry alias: {alias}"));
        }
        if !valid_mirror_authority(&registry.host) {
          return Err(format!("invalid mirror registry host: {}", registry.host));
        }
        if !mirror.allowed_hosts.contains(&registry.host) {
          return Err(format!(
            "mirror registry host is not in the outbound allowlist: {}",
            registry.host
          ));
        }
      }
      for host in &mirror.allowed_hosts {
        if !valid_mirror_authority(host) {
          return Err(format!("invalid mirror outbound host: {host}"));
        }
      }
    }
    Ok(())
  }
}

fn valid_mirror_authority(value: &str) -> bool {
  let Ok(url) = reqwest::Url::parse(&format!("https://{value}")) else {
    return false;
  };
  let Some(host) = url.host_str() else {
    return false;
  };
  let normalized = match url.port() {
    Some(443) | None => host.to_ascii_lowercase(),
    Some(port) => format!("{}:{port}", host.to_ascii_lowercase()),
  };
  url.path() == "/"
    && url.query().is_none()
    && url.fragment().is_none()
    && url.username().is_empty()
    && url.password().is_none()
    && normalized == value
}

impl MirrorSettings {
  pub fn permits_operation(&self, operation: crate::OciOperation) -> bool {
    matches!(
      operation,
      crate::OciOperation::Probe | crate::OciOperation::Pull | crate::OciOperation::Catalog
    )
  }
}
