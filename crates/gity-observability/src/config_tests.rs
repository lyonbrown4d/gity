use super::ObservabilityConfig;
#[test]
fn validates_operator_configuration_without_echoing_secret_values() {
  let mut config = ObservabilityConfig::for_service("registry");
  assert!(config.validate().is_ok());
  config.otlp_endpoint =
    "https://operator:private-test-secret@collector.example.test/v1/traces".into();
  let error = config.validate().unwrap_err().to_string();
  assert!(!error.contains("private-test-secret"));
  for value in [
    "file:///tmp/collector",
    "http://collector.example.test/v1/traces?token=private-test-secret",
  ] {
    config.otlp_endpoint = value.into();
    assert!(config.validate().is_err());
  }
  config.otlp_endpoint = "http://collector.example.test:4318/v1/traces".into();
  config.metrics_address = "invalid".into();
  assert!(config.validate().is_err());
  config.metrics_enabled = false;
  assert!(config.validate().is_ok());
  config.log_format = "invalid".into();
  assert!(config.validate().is_err());
}
