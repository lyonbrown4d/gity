use crate::CollaborationService;
use gity_core::AppError;
use gity_entities::{
  project_issue_assignees as assignees, project_merge_request_approvals as approvals,
  project_merge_request_participants as participants,
};

impl CollaborationService {
  pub async fn issue_assignees(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<assignees::Model>, AppError> {
    self.repository.issue_assignees(project, iid).await
  }
  pub async fn assigned_issues(
    &self,
    project: i64,
    issue_ids: Vec<i64>,
  ) -> Result<Vec<assignees::Model>, AppError> {
    self.repository.assigned_issues(project, issue_ids).await
  }
  pub async fn replace_assignees(
    &self,
    project: i64,
    iid: i64,
    users: Vec<i64>,
  ) -> Result<(), AppError> {
    self.repository.replace_assignees(project, iid, users).await
  }
  pub async fn merge_participants(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<participants::Model>, AppError> {
    self.repository.merge_participants(project, iid).await
  }
  pub async fn replace_participants(
    &self,
    project: i64,
    iid: i64,
    role: String,
    users: Vec<i64>,
  ) -> Result<(), AppError> {
    self
      .repository
      .replace_participants(project, iid, role, users)
      .await
  }
  pub async fn merge_approvals(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<approvals::Model>, AppError> {
    self.repository.merge_approvals(project, iid).await
  }
  pub async fn approve(
    &self,
    project: i64,
    iid: i64,
    user: i64,
    approved: bool,
  ) -> Result<(), AppError> {
    self.repository.approve(project, iid, user, approved).await
  }
}
