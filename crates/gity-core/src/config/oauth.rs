use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct OAuthConfig {
  pub authorization_code_ttl_seconds: u64,
  pub access_token_ttl_seconds: u64,
  pub refresh_token_ttl_seconds: u64,
  pub pending_request_ttl_seconds: u64,
  pub max_dynamic_clients: usize,
}

impl Default for OAuthConfig {
  fn default() -> Self {
    Self {
      authorization_code_ttl_seconds: 600,
      access_token_ttl_seconds: 3600,
      refresh_token_ttl_seconds: 2_592_000,
      pending_request_ttl_seconds: 300,
      max_dynamic_clients: 1000,
    }
  }
}

impl OAuthConfig {
  pub fn validate(&self) -> bool {
    (60..=900).contains(&self.authorization_code_ttl_seconds)
      && (60..=86_400).contains(&self.access_token_ttl_seconds)
      && (300..=31_536_000).contains(&self.refresh_token_ttl_seconds)
      && (60..=900).contains(&self.pending_request_ttl_seconds)
      && (1..=100_000).contains(&self.max_dynamic_clients)
  }
}
