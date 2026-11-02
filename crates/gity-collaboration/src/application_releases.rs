use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_releases as entity;

impl CollaborationService {
  pub async fn release(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    self.repository.release(project, key).await
  }
  pub async fn releases(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.releases(project).await
  }
  pub async fn create_release(&self, command: ReleaseCreate) -> Result<entity::Model, AppError> {
    if self
      .repository
      .find_release(command.project_id, &command.tag_name)
      .await?
      .is_some()
    {
      return Err(AppError::conflict("release already exists for tag"));
    }
    self.repository.create_release(command).await
  }
  pub async fn update_release(
    &self,
    project: i64,
    key: i64,
    changes: ReleaseChanges,
  ) -> Result<entity::Model, AppError> {
    self.repository.update_release(project, key, changes).await
  }
  pub async fn find_release(
    &self,
    project: i64,
    tag: &str,
  ) -> Result<Option<entity::Model>, AppError> {
    self.repository.find_release(project, tag).await
  }
  pub async fn delete_release(&self, project: i64, id: i64) -> Result<(), AppError> {
    self.repository.delete_release(project, id).await
  }
}
