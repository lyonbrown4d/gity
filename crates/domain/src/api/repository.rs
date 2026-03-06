use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Default, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct ListRepositoriesQuery {
  pub organization_id: Option<String>,
  pub all: Option<bool>,
  pub ids: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateRepositoryRequest {
  pub organization_id: Option<String>,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: Option<String>,
  pub default_branch: Option<String>,
  pub gitignore_template: Option<String>,
  pub license_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct RepositoryView {
  pub id: String,
  pub organization_id: String,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
  pub clone_http_url: String,
}
