use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiTransport {
  Rest,
  Graphql,
  Mcp,
}

/// Per-request credentials are revalidated by the application port. Never log them.
#[derive(Clone)]
pub struct ApiContext {
  authorization: Option<String>,
  job_token: Option<String>,
  pub language: String,
  pub request_id: String,
  pub transport: ApiTransport,
}
impl ApiContext {
  pub fn new(
    authorization: Option<String>,
    language: String,
    request_id: String,
    transport: ApiTransport,
  ) -> Self {
    Self {
      authorization,
      job_token: None,
      language,
      request_id,
      transport,
    }
  }
  /// Dedicated runner credential; never projected into GraphQL or MCP inputs.
  pub fn with_job_token(mut self, token: Option<String>) -> Self {
    self.job_token = token;
    self
  }
  pub fn job_token(&self) -> Option<&str> {
    self.job_token.as_deref()
  }
  pub fn authorization(&self) -> Option<&str> {
    self.authorization.as_deref()
  }
}
impl std::fmt::Debug for ApiContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ApiContext")
      .field(
        "authorization",
        &self.authorization.as_ref().map(|_| "[REDACTED]"),
      )
      .field("job_token", &self.job_token.as_ref().map(|_| "[REDACTED]"))
      .field("language", &self.language)
      .field("request_id", &self.request_id)
      .field("transport", &self.transport)
      .finish()
  }
}
