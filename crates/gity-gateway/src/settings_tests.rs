use super::GatewaySettings;

#[test]
fn rejects_untrusted_plain_http_and_accepts_explicit_private_network() {
  let mut settings = defaults();
  settings.main_http_origin = "http://gity-server:8080".into();
  assert!(settings.validate().is_err());
  settings.allow_insecure_private_network = true;
  assert!(settings.validate().is_ok());
}

#[test]
fn rejects_upstream_credentials_paths_and_unbounded_limits() {
  let mut settings = defaults();
  settings.main_http_origin = "https://user:secret@gity-server/api".into();
  assert!(settings.validate().is_err());
  settings = defaults();
  settings.main_ssh_address = "user@gity-server:2222".into();
  assert!(settings.validate().is_err());
  settings = defaults();
  settings.max_http_requests = 0;
  assert!(settings.validate().is_err());
}

fn defaults() -> GatewaySettings {
  GatewaySettings {
    http_listen: "127.0.0.1:0".parse().unwrap(),
    ssh_listen: "127.0.0.1:0".parse().unwrap(),
    package_http_origin: None,
    public_scheme: "http".into(),
    main_http_origin: "https://gity-server".into(),
    main_ssh_address: "gity-server:2222".into(),
    allow_insecure_private_network: false,
    request_timeout_seconds: 300,
    ssh_connect_timeout_seconds: 10,
    ssh_session_timeout_seconds: 3600,
    shutdown_timeout_seconds: 10,
    max_http_requests: 1024,
    max_ssh_sessions: 256,
  }
}
