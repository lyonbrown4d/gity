use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_issues as entity;

impl CollaborationService {
  pub async fn issue(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    self.repository.issue(project, key).await
  }
  pub async fn issues(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.issues(project).await
  }
  pub async fn create_issue(&self, command: IssueCreate) -> Result<entity::Model, AppError> {
    if !nonempty(&command.title) {
      return Err(AppError::bad_request("title is required"));
    }
    if let Some(id) = command.milestone_id {
      self.repository.milestone_id(command.project_id, id).await?;
    }
    self.repository.create_issue(command).await
  }
  pub async fn update_issue(
    &self,
    project: i64,
    key: i64,
    mut changes: IssueChanges,
  ) -> Result<entity::Model, AppError> {
    let current = self.repository.issue(project, key).await?;
    if let Change::Set(title) = &changes.title
      && !nonempty(title)
    {
      return Err(AppError::bad_request("title is required"));
    }
    if let Change::Set(Some(id)) = changes.milestone_id {
      self.repository.milestone_id(project, id).await?;
    }
    if let Change::Set(state) = &changes.state {
      let next = IssueState::parse(state).map_err(AppError::bad_request)?;
      let actor = match changes.updated_by_user_id {
        Change::Set(actor) => actor,
        Change::Unchanged => None,
      };
      if next == IssueState::Closed && current.state != "closed" {
        changes.closed_at = Change::Set(Some(chrono::Utc::now().naive_utc()));
        changes.closed_by_user_id = Change::Set(actor);
      } else if next == IssueState::Opened && current.state == "closed" {
        changes.closed_at = Change::Set(None);
        changes.closed_by_user_id = Change::Set(None);
      }
    }
    self.repository.update_issue(project, key, changes).await
  }
}
