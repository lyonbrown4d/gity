use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_issue_attachments as entity;

impl CollaborationService {
  pub async fn create_attachment(
    &self,
    project: i64,
    key: i64,
    mut command: AttachmentCreate,
  ) -> Result<entity::Model, AppError> {
    command.project_issue_id = self.repository.issue(project, key).await?.id;
    self.repository.create_attachment(command).await
  }
  pub async fn attachments(&self, project: i64, iid: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.attachments(project, iid).await
  }
  pub async fn attachment(
    &self,
    project: i64,
    iid: i64,
    id: i64,
  ) -> Result<entity::Model, AppError> {
    self.repository.attachment(project, iid, id).await
  }
}
