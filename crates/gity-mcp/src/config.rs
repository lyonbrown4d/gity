use std::collections::BTreeSet;
use std::time::Duration;
use url::Url;

#[derive(Clone, Debug)]
pub struct McpConfig {
  pub oauth_resource_metadata_url: String,
  pub allowed_origins: BTreeSet<String>,
  pub tool_groups: Option<BTreeSet<String>>,
  pub max_request_body_bytes: usize,
  pub max_concurrent_requests: usize,
  pub request_timeout: Duration,
  pub allowed_hosts: Vec<String>,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum McpConfigError {
  #[error("OAuth protected-resource metadata URL must be an absolute HTTP(S) URL")]
  InvalidMetadataUrl,
  #[error("allowed Origin must be an exact HTTP(S) origin without path, query, or fragment")]
  InvalidOrigin,
  #[error("MCP request limits must be greater than zero")]
  InvalidLimit,
  #[error("MCP operations must have unique names and object input schemas")]
  InvalidOperationContract,
}

impl Default for McpConfig {
  fn default() -> Self {
    Self {
      oauth_resource_metadata_url: "http://localhost/.well-known/oauth-protected-resource/mcp"
        .into(),
      allowed_origins: BTreeSet::new(),
      tool_groups: None,
      max_request_body_bytes: 4 * 1024 * 1024,
      max_concurrent_requests: 64,
      request_timeout: Duration::from_secs(30),
      allowed_hosts: vec!["localhost".into(), "127.0.0.1".into(), "[::1]".into()],
    }
  }
}

impl McpConfig {
  pub fn validate(&self) -> Result<(), McpConfigError> {
    let metadata = Url::parse(&self.oauth_resource_metadata_url)
      .map_err(|_| McpConfigError::InvalidMetadataUrl)?;
    if !matches!(metadata.scheme(), "http" | "https")
      || metadata.host_str().is_none()
      || metadata.username() != ""
      || metadata.password().is_some()
      || metadata.query().is_some()
      || metadata.fragment().is_some()
    {
      return Err(McpConfigError::InvalidMetadataUrl);
    }
    for origin in &self.allowed_origins {
      let parsed = Url::parse(origin).map_err(|_| McpConfigError::InvalidOrigin)?;
      if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.username() != ""
        || parsed.password().is_some()
      {
        return Err(McpConfigError::InvalidOrigin);
      }
    }
    if self.max_request_body_bytes == 0
      || self.max_concurrent_requests == 0
      || self.request_timeout.is_zero()
    {
      return Err(McpConfigError::InvalidLimit);
    }
    Ok(())
  }
}
