use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_issue_comments as entity;

impl CollaborationService {
  pub async fn create_issue_comment(
    &self,
    project: i64,
    key: i64,
    mut command: IssueCommentCreate,
  ) -> Result<entity::Model, AppError> {
    let issue = self.repository.issue(project, key).await?;
    command.project_issue_id = issue.id;
    command.position = crate::application_discussions::validate_position(command.position)?;
    let target = DiscussionTarget::Issue;
    command.discussion_id = Some(
      self
        .comment_discussion(project, target, issue.id, command.discussion_id)
        .await?,
    );
    self.repository.create_issue_comment(command).await
  }
  pub async fn issue_comments(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
    self.repository.issue_comments(project, iid).await
  }
  pub async fn issue_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
  ) -> Result<entity::Model, AppError> {
    self.repository.issue_comment(project, iid, comment).await
  }
  pub async fn update_issue_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
    command: IssueCommentUpdate,
  ) -> Result<entity::Model, AppError> {
    self
      .repository
      .update_issue_comment(project, iid, comment, command)
      .await
  }
  pub async fn delete_issue_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
  ) -> Result<(), AppError> {
    self
      .repository
      .delete_issue_comment(project, iid, comment)
      .await
  }
}
