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
    command.project_issue_id = self.repository.issue(project, key).await?.id;
    self.repository.create_issue_comment(command).await
  }
  pub async fn issue_comments(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
    self.repository.issue_comments(project, iid).await
  }
}
