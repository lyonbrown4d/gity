use crate::CollaborationService;
use gity_core::AppError;
use gity_entities::{project_branch_protections, project_pipelines};

impl CollaborationService {
  pub async fn validate_users(&self, ids: &[i64]) -> Result<(), AppError> {
    self.repository.validate_users(ids).await
  }
  pub async fn username_id(&self, name: &str) -> Result<Option<i64>, AppError> {
    self.repository.username_id(name).await
  }
  pub async fn recipients(&self, project: i64) -> Result<Vec<i64>, AppError> {
    self.repository.recipients(project).await
  }
  pub async fn protections(
    &self,
    project: i64,
  ) -> Result<Vec<project_branch_protections::Model>, AppError> {
    self.repository.protections(project).await
  }
  pub async fn source_pipeline(
    &self,
    project: i64,
    branch: &str,
    sha: &str,
  ) -> Result<Option<project_pipelines::Model>, AppError> {
    self.repository.source_pipeline(project, branch, sha).await
  }
}
