use std::{net::SocketAddr, path::PathBuf};

/// Private authority listener configuration. Secret fields intentionally do
/// not implement Debug.
#[derive(Clone)]
pub struct AuthoritySettings {
  pub address: SocketAddr,
  pub peer_token: String,
  pub tls_certificate: Option<PathBuf>,
  pub tls_key: Option<PathBuf>,
  pub client_ca: Option<PathBuf>,
}

impl AuthoritySettings {
  pub fn validate(&self) -> Result<(), String> {
    if self.peer_token.len() < 32 {
      return Err("authority peer token must be at least 32 bytes".into());
    }
    let has_tls =
      self.tls_certificate.is_some() && self.tls_key.is_some() && self.client_ca.is_some();
    if !self.address.ip().is_loopback() && !has_tls {
      return Err("non-loopback authority listeners require mutual TLS".into());
    }
    let any_tls =
      self.tls_certificate.is_some() || self.tls_key.is_some() || self.client_ca.is_some();
    if any_tls && !has_tls {
      return Err("authority mutual TLS paths must be configured together".into());
    }
    Ok(())
  }
}
