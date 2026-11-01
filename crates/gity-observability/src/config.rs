use serde::{Deserialize, Serialize};
#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ObservabilityConfig {
  pub service_name: String,
  pub log_format: String,
  pub log_level: String,
  pub metrics_enabled: bool,
  pub metrics_address: String,
  pub otlp_endpoint: String,
}
impl Default for ObservabilityConfig {
  fn default() -> Self {
    Self::for_service("gity")
  }
}
impl ObservabilityConfig {
  pub fn for_service(name: &str) -> Self {
    Self {
      service_name: name.into(),
      log_format: "json".into(),
      log_level: "gity=info,tower_http=info".into(),
      metrics_enabled: true,
      metrics_address: "127.0.0.1:9090".into(),
      otlp_endpoint: String::new(),
    }
  }
  pub fn validate(&self) -> anyhow::Result<()> {
    anyhow::ensure!(
      !self.service_name.trim().is_empty(),
      "observability service name is required"
    );
    anyhow::ensure!(
      matches!(self.log_format.as_str(), "json" | "text"),
      "observability log format must be json or text"
    );
    if self.metrics_enabled {
      self
        .metrics_address
        .parse::<std::net::SocketAddr>()
        .map_err(|_| anyhow::anyhow!("invalid observability metrics address"))?;
    }
    if !self.otlp_endpoint.is_empty() {
      let url = url::Url::parse(&self.otlp_endpoint)
        .map_err(|_| anyhow::anyhow!("invalid OTLP endpoint"))?;
      anyhow::ensure!(
        matches!(url.scheme(), "http" | "https")
          && url.host_str().is_some()
          && url.username().is_empty()
          && url.password().is_none()
          && url.query().is_none()
          && url.fragment().is_none(),
        "OTLP endpoint must be HTTP(S) without user credentials"
      );
    }
    Ok(())
  }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
