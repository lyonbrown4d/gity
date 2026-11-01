use super::defaults;
use crate::Args;
use config::{Config, File, FileFormat};

#[test]
fn runner_observability_defaults_are_isolated_and_metrics_are_opt_in() {
  let args: Args = Config::builder()
    .add_source(File::from_str(&defaults().to_string(), FileFormat::Json))
    .build()
    .unwrap()
    .try_deserialize()
    .unwrap();

  assert_eq!(args.observability.service_name, "gity-runner");
  assert_eq!(args.observability.metrics_address, "127.0.0.1:9091");
  assert!(!args.observability.metrics_enabled);
}

#[test]
fn runner_observability_file_values_override_defaults() {
  let args: Args = Config::builder()
    .add_source(File::from_str(&defaults().to_string(), FileFormat::Json))
    .add_source(File::from_str(
      "[observability]\nmetrics_enabled = true\nmetrics_address = '127.0.0.1:19091'\nservice_name = 'runner-acceptance'\n",
      FileFormat::Toml,
    ))
    .build()
    .unwrap()
    .try_deserialize()
    .unwrap();

  assert_eq!(args.observability.service_name, "runner-acceptance");
  assert_eq!(args.observability.metrics_address, "127.0.0.1:19091");
  assert!(args.observability.metrics_enabled);
}
