pub fn init_logging() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_test_writer()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
}
