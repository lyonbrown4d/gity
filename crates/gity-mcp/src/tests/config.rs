use crate::{McpConfig, McpConfigError};
use std::collections::BTreeSet;

#[test]
fn validates_metadata_origin_and_limits() {
  let mut config = McpConfig::default();
  assert!(config.validate().is_ok());
  config.allowed_origins.insert("https://example.test".into());
  assert!(config.validate().is_ok());
  config
    .allowed_origins
    .insert("https://example.test/path".into());
  assert!(matches!(
    config.validate(),
    Err(McpConfigError::InvalidOrigin)
  ));
  config.allowed_origins = BTreeSet::new();
  config.max_concurrent_requests = 0;
  assert!(matches!(
    config.validate(),
    Err(McpConfigError::InvalidLimit)
  ));
}
