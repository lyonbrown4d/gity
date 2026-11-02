use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_merge_request_comments as entity;

impl CollaborationService {
  pub async fn create_merge_comment(
    &self,
    project: i64,
    key: i64,
    mut command: MergeCommentCreate,
  ) -> Result<entity::Model, AppError> {
    command.merge_request_id = self.repository.merge_request(project, key).await?.id;
    self.repository.create_merge_comment(command).await
  }
  pub async fn merge_comments(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
    self.repository.merge_comments(project, iid).await
  }
}
