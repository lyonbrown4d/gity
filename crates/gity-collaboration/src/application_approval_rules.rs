use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_merge_request_approval_rules as entity;

impl CollaborationService {
  pub async fn approval_rule(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    self.repository.approval_rule(project, key).await
  }
  pub async fn approval_rules(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.approval_rules(project).await
  }
  pub async fn create_approval_rule(
    &self,
    command: ApprovalRuleCreate,
    actor: Option<i64>,
  ) -> Result<entity::Model, AppError> {
    self.repository.create_approval_rule(command, actor).await
  }
  pub async fn update_approval_rule(
    &self,
    project: i64,
    key: i64,
    changes: ApprovalRuleChanges,
    actor: Option<i64>,
  ) -> Result<entity::Model, AppError> {
    self
      .repository
      .update_approval_rule(project, key, changes, actor)
      .await
  }
  pub async fn delete_approval_rule(
    &self,
    project: i64,
    id: i64,
    actor: Option<i64>,
  ) -> Result<(), AppError> {
    self
      .repository
      .delete_approval_rule(project, id, actor)
      .await
  }
}
