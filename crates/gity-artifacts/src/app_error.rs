use crate::ArtifactValidationError;

impl From<ArtifactValidationError> for gity_core::AppError {
  fn from(error: ArtifactValidationError) -> Self {
    Self::bad_request(error.message())
  }
}
