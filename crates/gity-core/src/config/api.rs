use serde::Deserialize;

/// Bounds apply equally to standalone and independently deployed API processes.
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
  pub graphql_enabled: bool,
  pub mcp_enabled: bool,
  pub graphql_introspection: bool,
  pub graphql_max_depth: usize,
  pub graphql_max_complexity: usize,
  pub max_request_bytes: usize,
  pub mcp_max_concurrent_requests: usize,
  pub mcp_request_timeout_seconds: u64,
  pub mcp_allowed_origins: Vec<String>,
  pub mcp_allowed_hosts: Vec<String>,
}
impl Default for ApiConfig {
  fn default() -> Self {
    Self {
      graphql_enabled: true,
      mcp_enabled: true,
      graphql_introspection: false,
      graphql_max_depth: 12,
      graphql_max_complexity: 200,
      max_request_bytes: 1024 * 1024,
      mcp_max_concurrent_requests: 64,
      mcp_request_timeout_seconds: 30,
      mcp_allowed_origins: Vec::new(),
      mcp_allowed_hosts: Vec::new(),
    }
  }
}
impl ApiConfig {
  pub fn validate(&self) -> Result<(), crate::AppError> {
    if !(1..=64).contains(&self.graphql_max_depth)
      || !(1..=10_000).contains(&self.graphql_max_complexity)
      || !(1..=16 * 1024 * 1024).contains(&self.max_request_bytes)
      || !(1..=1024).contains(&self.mcp_max_concurrent_requests)
      || !(1..=300).contains(&self.mcp_request_timeout_seconds)
    {
      return Err(crate::AppError::bad_request("invalid API request limits"));
    }
    Ok(())
  }
}
