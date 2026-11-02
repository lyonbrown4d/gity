use std::num::NonZeroUsize;

/// Limits for the GraphQL endpoint. Defaults are bounded and disable introspection.
#[derive(Clone, Debug)]
pub struct GraphqlConfig {
  pub max_depth: NonZeroUsize,
  pub max_complexity: NonZeroUsize,
  pub max_request_bytes: usize,
  pub introspection_enabled: bool,
}

impl Default for GraphqlConfig {
  fn default() -> Self {
    Self {
      max_depth: NonZeroUsize::new(12).expect("nonzero default"),
      max_complexity: NonZeroUsize::new(200).expect("nonzero default"),
      max_request_bytes: 1_048_576,
      introspection_enabled: false,
    }
  }
}

impl GraphqlConfig {
  pub fn validate(&self) -> Result<(), String> {
    if self.max_request_bytes == 0 {
      return Err("graphql max_request_bytes must be positive".into());
    }
    if self.max_depth.get() > 64 || self.max_complexity.get() > 10_000 {
      return Err("graphql depth and complexity limits exceed the hard maximum".into());
    }
    if self.max_request_bytes > 16 * 1024 * 1024 {
      return Err("graphql max_request_bytes exceeds 16 MiB".into());
    }
    Ok(())
  }
}
