use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct ArtifactValidationError {
  message: &'static str,
}

impl ArtifactValidationError {
  pub const fn new(message: &'static str) -> Self {
    Self { message }
  }

  pub const fn message(self) -> &'static str {
    self.message
  }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
