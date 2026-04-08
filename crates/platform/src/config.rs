use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
  pub url: String,
}

impl DatabaseConfig {
  pub fn new(url: impl Into<String>) -> Self {
    Self { url: url.into() }
  }
}
