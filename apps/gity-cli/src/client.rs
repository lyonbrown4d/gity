//! Resolved credentials plus a ready-to-use API client.
//!
//! The HTTP layer lives in `gity-client`; this module only turns CLI options and
//! the stored configuration into a client instance.
use anyhow::Result;
pub use gity_client::Client;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Options {
  pub host: Option<String>,
  pub token: Option<String>,
  pub json: bool,
  pub config_file: PathBuf,
}

impl Options {
  pub fn client(&self) -> Result<Client> {
    let resolved = crate::config::resolve(self)?;
    Ok(Client::new(&resolved.host, &resolved.token))
  }
}
