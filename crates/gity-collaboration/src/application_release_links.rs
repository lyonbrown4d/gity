use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_release_links as entity;

impl CollaborationService {
  pub async fn create_release_link(
    &self,
    project: i64,
    key: i64,
    mut command: ReleaseLinkCreate,
  ) -> Result<entity::Model, AppError> {
    command.project_release_id = self.repository.release(project, key).await?.id;
    self.repository.create_release_link(command).await
  }
  pub async fn release_links(&self, project: i64, id: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.release_links(project, id).await
  }
  pub async fn delete_release_link(
    &self,
    project: i64,
    release: i64,
    id: i64,
  ) -> Result<(), AppError> {
    self
      .repository
      .delete_release_link(project, release, id)
      .await
  }
}
