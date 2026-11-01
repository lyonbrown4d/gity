use super::{RegistryOverrides, RegistrySettings};
use std::{io::Write, time::Duration};

#[tokio::test]
async fn loads_typed_registry_config_and_applies_defaults() {
  let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
  file
    .write_all(
      br#"
address = "127.0.0.1:8081"

[angos]
config_files = ["/tmp/angos-test.toml"]
health_url = "http://127.0.0.1:8000/readyz"

[authority]
uri = "http://127.0.0.1:9102"

[oci]
upstream = "http://127.0.0.1:8000"
"#,
    )
    .unwrap();
  let settings = RegistrySettings::load(Some(file.path()), RegistryOverrides::default()).unwrap();
  let runtime = settings.into_runtime("p".repeat(32)).await.unwrap();
  assert_eq!(runtime.address.port(), 8081);
  assert_eq!(
    runtime.angos.executable.to_str().unwrap(),
    "/usr/local/bin/angos"
  );
  assert_eq!(runtime.oci.request_timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn credentials_load_from_file_and_explicit_cli_wins() {
  let mut credential = tempfile::NamedTempFile::new().unwrap();
  writeln!(credential, "{}", "f".repeat(32)).unwrap();
  let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
  writeln!(
    file,
    "[angos]\nconfig_files = [\"/tmp/angos-test.toml\"]\n[authority]\npeer_token = \"{}\"\ntoken_file = {:?}",
    "c".repeat(32),
    credential.path().to_str().unwrap()
  )
  .unwrap();
  let load = || RegistrySettings::load(Some(file.path()), RegistryOverrides::default()).unwrap();
  assert_eq!(
    load()
      .into_runtime(String::new())
      .await
      .unwrap()
      .authority_endpoint
      .token,
    "f".repeat(32)
  );
  assert_eq!(
    load()
      .into_runtime("p".repeat(32))
      .await
      .unwrap()
      .authority_endpoint
      .token,
    "p".repeat(32)
  );
  std::fs::remove_file(credential.path()).unwrap();
  assert!(load().into_runtime(String::new()).await.is_err());
  assert!(load().into_runtime("p".repeat(32)).await.is_ok());
}

#[tokio::test]
async fn configuration_accepts_peer_token_and_rejects_short_credentials_without_echoing() {
  let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
  writeln!(
    file,
    "[angos]\nconfig_files = [\"/tmp/angos-test.toml\"]\n[authority]\npeer_token = \"{}\"",
    "c".repeat(32)
  )
  .unwrap();
  let settings = RegistrySettings::load(Some(file.path()), RegistryOverrides::default()).unwrap();
  assert_eq!(
    settings
      .into_runtime(String::new())
      .await
      .unwrap()
      .authority_endpoint
      .token,
    "c".repeat(32)
  );
  let settings = RegistrySettings::load(Some(file.path()), RegistryOverrides::default()).unwrap();
  let error = settings
    .into_runtime("short-secret".into())
    .await
    .err()
    .unwrap();
  assert!(error.to_string().contains("authority peer token"));
  assert!(!error.to_string().contains("short-secret"));
}

#[tokio::test]
async fn explicit_cli_overrides_take_precedence_over_file_values() {
  let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
  file
    .write_all(
      br#"
address = "127.0.0.1:8081"

[angos]
config_files = ["/tmp/angos.toml"]

[authority]
uri = "http://file-authority:9102"

[oci]
upstream = "http://file-angos:8000"
"#,
    )
    .unwrap();
  let settings = RegistrySettings::load(
    Some(file.path()),
    RegistryOverrides {
      address: Some("127.0.0.1:9090".parse().unwrap()),
      request_timeout_seconds: Some(45),
      angos_executable: Some("/custom/angos".into()),
      authority_uri: Some("http://cli-authority:9102".into()),
      oci_upstream: Some("http://cli-angos:8000".into()),
    },
  )
  .unwrap();
  let runtime = settings.into_runtime("p".repeat(32)).await.unwrap();
  assert_eq!(runtime.address.port(), 9090);
  assert_eq!(runtime.angos.executable.to_str().unwrap(), "/custom/angos");
  assert_eq!(runtime.authority_endpoint.uri, "http://cli-authority:9102");
  assert_eq!(runtime.oci.upstream, "http://cli-angos:8000");
  assert_eq!(runtime.oci.request_timeout, Duration::from_secs(45));
}
