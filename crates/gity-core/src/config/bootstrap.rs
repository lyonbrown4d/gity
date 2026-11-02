use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct BootstrapConfig {
  pub enabled: bool,
  pub username: String,
  pub email: Option<String>,
  pub initial_password_file: Option<PathBuf>,
  pub generated_password_file: Option<PathBuf>,
}

impl Default for BootstrapConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      username: "root".to_owned(),
      email: None,
      initial_password_file: None,
      generated_password_file: None,
    }
  }
}

impl BootstrapConfig {
  pub(crate) fn with_state_root(mut self, state_root: &Path) -> Self {
    if self.generated_password_file.is_none() {
      self.generated_password_file = Some(state_root.join("bootstrap/admin-password"));
    }
    self
  }
}
