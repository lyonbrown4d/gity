use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectCommand {
  pub namespace_id: i64,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: Option<String>,
  pub actor_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectView {
  pub id: i64,
  pub namespace_id: i64,
  pub full_path: String,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
  pub archived: bool,
  pub created_by_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectResult {
  pub project_id: i64,
  pub full_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectBranchCommand {
  pub project_id: i64,
  pub name: String,
  pub source_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetProjectBranchProtectionCommand {
  pub project_id: i64,
  pub branch_name: String,
  pub is_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBranchView {
  pub id: i64,
  pub project_id: i64,
  pub name: String,
  pub is_protected: bool,
  pub last_commit_sha: Option<String>,
  pub created_at_unix: i64,
  pub updated_at_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLanguageSnapshotItemView {
  pub language: String,
  pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLanguageSnapshotView {
  pub project_id: i64,
  pub branch_name: String,
  pub revision: String,
  pub analyzed_at_unix: i64,
  pub total_bytes: u64,
  pub items: Vec<ProjectLanguageSnapshotItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectIssueCommand {
  pub project_id: i64,
  pub title: String,
  pub description: Option<String>,
  pub assignee_user_id: Option<String>,
  pub author_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectIssueCommand {
  pub project_id: i64,
  pub issue_id: i64,
  pub title: Option<String>,
  pub description: Option<Option<String>>,
  pub state: Option<String>,
  pub assignee_user_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssueView {
  pub id: i64,
  pub project_id: i64,
  pub iid: i64,
  pub title: String,
  pub description: Option<String>,
  pub state: String,
  pub author_user_id: String,
  pub assignee_user_id: Option<String>,
  pub created_at_unix: i64,
  pub updated_at_unix: i64,
  pub closed_at_unix: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectIssueCommentCommand {
  pub project_id: i64,
  pub issue_id: i64,
  pub body: String,
  pub author_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssueCommentView {
  pub id: i64,
  pub project_issue_id: i64,
  pub author_user_id: String,
  pub body: String,
  pub created_at_unix: i64,
  pub updated_at_unix: i64,
}

pub trait ProjectLifecycle {
  fn service_name(&self) -> &'static str {
    "project-lifecycle"
  }
}
