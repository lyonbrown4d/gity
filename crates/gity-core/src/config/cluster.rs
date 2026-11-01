use crate::AppError;
use serde::Deserialize;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitClusterConfig {
  pub nodes: Vec<GitStorageNodeConfig>,
  #[serde(default = "one")]
  pub replicas: usize,
  #[serde(default = "five")]
  pub replication_interval_seconds: u64,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitStorageNodeConfig {
  pub id: String,
  pub endpoint: String,
  #[serde(default)]
  pub token: String,
  pub token_file: Option<PathBuf>,
  pub ca_file: Option<PathBuf>,
  pub client_certificate_file: Option<PathBuf>,
  pub client_key_file: Option<PathBuf>,
}
impl GitClusterConfig {
  pub fn load_credentials(&mut self) -> Result<(), AppError> {
    for node in &mut self.nodes {
      if let Some(path) = &node.token_file {
        let metadata = std::fs::metadata(path)
          .map_err(|_| AppError::bad_request("Git node credential file is unavailable"))?;
        if !metadata.is_file() || metadata.len() > 4096 {
          return Err(AppError::bad_request(
            "Git node credential file must be a small regular file",
          ));
        }
        node.token = std::fs::read_to_string(path)
          .map_err(|_| AppError::bad_request("Git node credential file is unreadable"))?
          .trim_end_matches(['\r', '\n'])
          .to_owned();
      }
    }
    Ok(())
  }
  pub fn validate(&self) -> Result<(), AppError> {
    if self.nodes.is_empty()
      || self.replicas == 0
      || self.replicas > self.nodes.len()
      || self.replication_interval_seconds == 0
    {
      return Err(AppError::bad_request(
        "Git cluster requires nodes, a valid replica count, and a positive replication interval",
      ));
    }
    let mut ids = BTreeSet::new();
    for node in &self.nodes {
      if node.id.is_empty() || !ids.insert(&node.id) || node.token.len() < 32 {
        return Err(AppError::bad_request(
          "Git node IDs must be unique and internal credentials must contain at least 32 bytes",
        ));
      }
    }
    Ok(())
  }
}
fn one() -> usize {
  1
}
fn five() -> u64 {
  5
}
