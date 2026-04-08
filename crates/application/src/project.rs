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

pub trait ProjectLifecycle {
  fn service_name(&self) -> &'static str {
    "project-lifecycle"
  }
}
