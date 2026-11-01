use super::*;
#[test]
fn explicit_yaml_and_toml_files_use_typed_defaults_and_validation() {
  let directory = tempfile::tempdir().unwrap();
  let yaml = directory.path().join("gity.yaml");
  std::fs::write(
    &yaml,
    "http:\n  address: ':8123'\nsearch:\n  max_files_per_project: 400\n",
  )
  .unwrap();
  let settings = Settings::try_load(Some(&yaml)).unwrap();
  assert_eq!(settings.address, "0.0.0.0:8123");
  assert_eq!(settings.search.max_files_per_project, 400);
  assert_eq!(settings.search.refresh_interval_seconds, 300);
  let toml = directory.path().join("gity.toml");
  std::fs::write(&toml, "[database]\nnode_id = 1024\n").unwrap();
  assert!(Settings::try_load(Some(&toml)).is_err());
  std::fs::write(&toml, "[search]\nrefresh_interval_seconds = 0\n").unwrap();
  assert!(Settings::try_load(Some(&toml)).is_err());
  std::fs::write(&toml, "[observability]\nlog_format = 'invalid'\n").unwrap();
  assert!(Settings::try_load(Some(&toml)).is_err());
  std::fs::write(
    &toml,
    "[database]\ndriver = 'postgres'\ndsn = 'sqlite::memory:'\n",
  )
  .unwrap();
  assert!(Settings::try_load(Some(&toml)).is_err());
  assert!(Settings::try_load(Some(&directory.path().join("missing.yaml"))).is_err());
}

#[test]
fn ssh_listener_uses_typed_configurable_port_and_bounded_limits() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("ssh.yaml");
  std::fs::write(&path, "ssh:\n  enabled: true\n  address: 127.0.0.1:2228\n  max_connections: 8\n  operation_timeout_seconds: 30\n").unwrap();
  let settings = Settings::try_load(Some(&path)).unwrap();
  assert!(settings.ssh.enabled);
  assert_eq!(settings.ssh.address, "127.0.0.1:2228");
  assert_eq!(settings.ssh.max_connections, 8);
  for body in [
    "ssh:\n  address: invalid\n",
    "ssh:\n  max_connections: 0\n",
    "ssh:\n  operation_timeout_seconds: 0\n",
  ] {
    std::fs::write(&path, body).unwrap();
    assert!(Settings::try_load(Some(&path)).is_err());
  }
}

#[test]
fn cluster_node_credentials_are_loaded_from_files_and_validated() {
  let dir = tempfile::tempdir().unwrap();
  let secret = dir.path().join("peer-token");
  std::fs::write(
    &secret,
    "a-private-test-peer-credential-at-least-32-bytes\n",
  )
  .unwrap();
  let config = dir.path().join("cluster.toml");
  let body = format!(
    r#"
[git.cluster]
replicas = 1
[[git.cluster.nodes]]
id = "one"
endpoint = "http://127.0.0.1:9010"
token_file = {:?}
"#,
    secret.to_str().unwrap()
  );
  std::fs::write(&config, body).unwrap();
  let loaded = Settings::try_load(Some(&config)).unwrap();
  assert_eq!(
    loaded.git_cluster.unwrap().nodes[0].token,
    "a-private-test-peer-credential-at-least-32-bytes"
  );
  std::fs::write(&secret, "short").unwrap();
  assert!(Settings::try_load(Some(&config)).is_err());
  std::fs::remove_file(&secret).unwrap();
  assert!(Settings::try_load(Some(&config)).is_err());
}
