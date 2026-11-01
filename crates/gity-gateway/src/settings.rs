use std::{net::SocketAddr, time::Duration};

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;

#[derive(Clone)]
pub struct GatewaySettings {
  pub http_listen: SocketAddr,
  pub ssh_listen: SocketAddr,
  pub main_http_origin: String,
  pub package_http_origin: Option<String>,
  pub public_scheme: String,
  pub main_ssh_address: String,
  pub allow_insecure_private_network: bool,
  pub request_timeout_seconds: u64,
  pub ssh_connect_timeout_seconds: u64,
  pub ssh_session_timeout_seconds: u64,
  pub shutdown_timeout_seconds: u64,
  pub max_http_requests: usize,
  pub max_ssh_sessions: usize,
}

impl GatewaySettings {
  pub fn validate(&self) -> Result<(), String> {
    if !matches!(self.public_scheme.as_str(), "http" | "https") {
      return Err("gateway public scheme must be http or https".into());
    }
    for value in std::iter::once(&self.main_http_origin).chain(self.package_http_origin.iter()) {
      let origin = url::Url::parse(value).map_err(|_| "HTTP origin must be a valid URL")?;
      let private_http = origin.scheme() == "http" && self.allow_insecure_private_network;
      if (origin.scheme() != "https" && !private_http)
        || origin.host_str().is_none()
        || !origin.username().is_empty()
        || origin.password().is_some()
        || origin.path() != "/"
        || origin.query().is_some()
        || origin.fragment().is_some()
      {
        return Err("HTTP origins must be HTTPS, or explicitly trusted private HTTP".into());
      }
    }
    let ssh = url::Url::parse(&format!("ssh://{}", self.main_ssh_address))
      .map_err(|_| "main SSH address must be host:port")?;
    if ssh.host_str().is_none()
      || ssh.port().is_none()
      || ssh.username() != ""
      || ssh.password().is_some()
      || !matches!(ssh.path(), "" | "/")
      || ssh.query().is_some()
      || ssh.fragment().is_some()
    {
      return Err("main SSH address must be host:port without credentials or a path".into());
    }
    if self.request_timeout_seconds == 0
      || self.ssh_connect_timeout_seconds == 0
      || self.ssh_session_timeout_seconds == 0
      || self.shutdown_timeout_seconds == 0
      || self.max_http_requests == 0
      || self.max_ssh_sessions == 0
    {
      return Err("gateway timeouts and concurrency limits must be positive".into());
    }
    Ok(())
  }

  pub(super) fn request_timeout(&self) -> Duration {
    Duration::from_secs(self.request_timeout_seconds)
  }

  pub(super) fn ssh_connect_timeout(&self) -> Duration {
    Duration::from_secs(self.ssh_connect_timeout_seconds)
  }

  pub(super) fn ssh_session_timeout(&self) -> Duration {
    Duration::from_secs(self.ssh_session_timeout_seconds)
  }

  pub(super) fn shutdown_timeout(&self) -> Duration {
    Duration::from_secs(self.shutdown_timeout_seconds)
  }
}
