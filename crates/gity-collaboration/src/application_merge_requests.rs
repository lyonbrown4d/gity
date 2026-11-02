use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_merge_requests as entity;

impl CollaborationService {
  pub async fn merge_request(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    self.repository.merge_request(project, key).await
  }
  pub async fn merge_requests(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.merge_requests(project).await
  }
  pub async fn create_merge_request(
    &self,
    command: MergeRequestCreate,
  ) -> Result<entity::Model, AppError> {
    if !nonempty(&command.title) {
      return Err(AppError::bad_request("title is required"));
    }
    if command.source_branch == command.target_branch {
      return Err(AppError::bad_request(
        "source_branch and target_branch must be different",
      ));
    }
    if let Some(id) = command.milestone_id {
      self.repository.milestone_id(command.project_id, id).await?;
    }
    self.repository.create_merge_request(command).await
  }
  pub async fn update_merge_request(
    &self,
    project: i64,
    key: i64,
    mut changes: MergeRequestChanges,
  ) -> Result<entity::Model, AppError> {
    let current = self.repository.merge_request(project, key).await?;
    if let Change::Set(title) = &changes.title
      && !nonempty(title)
    {
      return Err(AppError::bad_request("title is required"));
    }
    if let Change::Set(Some(id)) = changes.milestone_id {
      self.repository.milestone_id(project, id).await?;
    }
    if let Change::Set(state) = &changes.state {
      if current.state == "merged" {
        return Err(AppError::conflict(
          "merged merge requests cannot be reopened",
        ));
      }
      let next = MergeRequestState::editable_transition(&current.state, state)
        .map_err(AppError::bad_request)?;
      changes.closed_at =
        Change::Set((next == MergeRequestState::Closed).then(|| chrono::Utc::now().naive_utc()));
    }
    self
      .repository
      .update_merge_request(project, key, changes)
      .await
  }
}
