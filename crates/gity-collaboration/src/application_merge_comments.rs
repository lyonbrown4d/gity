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
    let merge_request = self.repository.merge_request(project, key).await?;
    command.merge_request_id = merge_request.id;
    command.position = crate::application_discussions::validate_position(command.position)?;
    let target = DiscussionTarget::MergeRequest;
    command.discussion_id = Some(
      self
        .comment_discussion(project, target, merge_request.id, command.discussion_id)
        .await?,
    );
    self.repository.create_merge_comment(command).await
  }
  pub async fn merge_comments(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
    self.repository.merge_comments(project, iid).await
  }
  pub async fn merge_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
  ) -> Result<entity::Model, AppError> {
    self.repository.merge_comment(project, iid, comment).await
  }
  pub async fn update_merge_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
    command: MergeCommentUpdate,
  ) -> Result<entity::Model, AppError> {
    self
      .repository
      .update_merge_comment(project, iid, comment, command)
      .await
  }
  pub async fn delete_merge_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
  ) -> Result<(), AppError> {
    self
      .repository
      .delete_merge_comment(project, iid, comment)
      .await
  }
}
