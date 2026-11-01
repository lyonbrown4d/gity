use super::{GatewayFile, GatewayOverrides};
use std::{io::Write, net::SocketAddr};

#[test]
fn explicit_cli_values_override_file_values() {
  let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
  file
    .write_all(
      br#"
http_listen = "127.0.0.1:8081"
ssh_listen = "127.0.0.1:2221"
main_http_origin = "https://gity-server"
main_ssh_address = "gity-server:2222"
"#,
    )
    .unwrap();
  let settings = GatewayFile::load(
    Some(file.path()),
    GatewayOverrides {
      http_listen: Some("127.0.0.1:8082".parse::<SocketAddr>().unwrap()),
      ..GatewayOverrides::default()
    },
  )
  .unwrap()
  .into_runtime()
  .unwrap();
  assert_eq!(settings.http_listen.port(), 8082);
  assert_eq!(settings.ssh_listen.port(), 2221);
}
