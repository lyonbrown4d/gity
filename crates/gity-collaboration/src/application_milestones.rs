use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_milestones as entity;

impl CollaborationService {
  pub async fn milestones(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.milestones(project).await
  }
  pub async fn milestone(&self, project: i64, iid: i64) -> Result<entity::Model, AppError> {
    self.repository.milestone(project, iid).await
  }
  pub async fn milestone_id(&self, project: i64, id: i64) -> Result<i64, AppError> {
    self.repository.milestone_id(project, id).await
  }
  pub async fn create_milestone(
    &self,
    project: i64,
    command: MilestoneCreate,
  ) -> Result<entity::Model, AppError> {
    self.repository.create_milestone(project, command).await
  }
  pub async fn update_milestone(
    &self,
    project: i64,
    iid: i64,
    changes: MilestoneChanges,
  ) -> Result<entity::Model, AppError> {
    self
      .repository
      .update_milestone(project, iid, changes)
      .await
  }
  pub async fn delete_milestone(&self, project: i64, iid: i64) -> Result<(), AppError> {
    self.repository.delete_milestone(project, iid).await
  }
}
