use serde::Deserialize;

/// Global search configuration. Remote credentials never enter API contracts.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GlobalSearchSettings {
  pub enabled: bool,
  pub backend: String,
  pub endpoint: String,
  pub index: String,
  pub username: String,
  pub password: String,
  pub timeout_seconds: u64,
  pub allow_insecure_http: bool,
}
impl Default for GlobalSearchSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      backend: "tantivy".into(),
      endpoint: String::new(),
      index: "gity-global-v1".into(),
      username: String::new(),
      password: String::new(),
      timeout_seconds: 10,
      allow_insecure_http: false,
    }
  }
}
impl GlobalSearchSettings {
  pub fn validate(&self) -> bool {
    matches!(self.backend.as_str(), "tantivy" | "opensearch")
      && (1..=60).contains(&self.timeout_seconds)
      && !self.index.is_empty()
      && self
        .index
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
      && (self.backend != "opensearch" || !self.endpoint.is_empty())
  }
}
