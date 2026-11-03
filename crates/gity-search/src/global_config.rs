use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
  #[default]
  Tantivy,
  OpenSearch,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct GlobalSearchConfig {
  pub backend: BackendKind,
  pub endpoint: String,
  pub index: String,
  pub username: String,
  pub password: String,
  pub timeout_seconds: u64,
  pub allow_insecure_http: bool,
}

impl Default for GlobalSearchConfig {
  fn default() -> Self {
    Self {
      backend: BackendKind::Tantivy,
      endpoint: String::new(),
      index: "gity-global-v1".to_owned(),
      username: String::new(),
      password: String::new(),
      timeout_seconds: 10,
      allow_insecure_http: false,
    }
  }
}

impl GlobalSearchConfig {
  pub(crate) fn validate(&self) -> Result<()> {
    if self.timeout_seconds == 0 || self.timeout_seconds > 300 {
      bail!("global search timeout must be between 1 and 300 seconds");
    }
    if self.index.is_empty()
      || self.index.len() > 255
      || !self
        .index
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
    {
      bail!("invalid global search index name");
    }
    if self.backend == BackendKind::OpenSearch {
      let url = reqwest::Url::parse(&self.endpoint)
        .map_err(|_| anyhow::anyhow!("invalid OpenSearch endpoint"))?;
      if !matches!(url.scheme(), "https" | "http")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !self.endpoint.is_ascii()
      {
        bail!("invalid OpenSearch endpoint");
      }
      if url.scheme() == "http"
        && !self.allow_insecure_http
        && !matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
      {
        bail!("OpenSearch HTTP is allowed only for local endpoints");
      }
    }
    Ok(())
  }

  pub(crate) fn timeout(&self) -> Duration {
    Duration::from_secs(self.timeout_seconds)
  }
}
