use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(Serialize, ToSchema)]
pub struct Organization {
  pub key: String,
  pub role: String,
  pub id: String,
  pub name: String,
  pub path_key: String,
  pub full_path: String,
  pub description: Option<String>,
  pub visibility: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateOrganization {
  pub key: Option<String>,
  pub name: String,
  pub path_key: Option<String>,
  pub path: Option<String>,
  pub description: Option<String>,
  pub visibility: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateOrganization {
  pub key: Option<String>,
  pub name: Option<String>,
  pub description: Option<String>,
  pub visibility: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct Project {
  pub key: String,
  pub uuid: String,
  pub clone_http_url: String,
  pub id: String,
  pub organization_id: String,
  pub name: String,
  pub path_key: String,
  pub full_path: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
  pub status: String,
  pub deleted_at: Option<String>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateProject {
  pub key: Option<String>,
  pub organization_id: i64,
  pub name: String,
  pub path_key: Option<String>,
  pub path: Option<String>,
  pub visibility: Option<String>,
  pub description: Option<String>,
  pub default_branch: Option<String>,
  /// Embedded license template ID; omitted or `none` creates no LICENSE.
  pub license_template: Option<String>,
  /// MIT/BSD copyright holder; defaults to the owning organization name.
  pub license_copyright_holder: Option<String>,
  /// Create README.md in the initial commit. Defaults to false.
  pub initialize_with_readme: Option<bool>,
  /// Embedded .gitignore template ID; omitted or `none` creates no .gitignore.
  pub gitignore_template: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateProject {
  pub name: Option<String>,
  pub description: Option<String>,
  pub visibility: Option<String>,
  pub default_branch: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct Permissions {
  /// Effective user role, or project_token for credentials governed only by scopes.
  pub role: String,
  pub actions: std::collections::BTreeMap<String, bool>,
  pub project_id: String,
  pub user_id: String,
  pub capabilities: std::collections::BTreeMap<String, bool>,
  pub role_level: i64,
}
#[derive(Serialize, ToSchema)]
pub struct ProjectMember {
  pub id: String,
  pub project_id: String,
  pub user_id: String,
  pub role: String,
  pub username: Option<String>,
  pub display_name: Option<String>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct OrganizationMember {
  pub email: Option<String>,
  pub id: String,
  pub organization_id: String,
  pub user_id: String,
  pub role: String,
  pub username: Option<String>,
  pub display_name: Option<String>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct AddMember {
  pub user_id: i64,
  pub role: String,
}
#[derive(Deserialize, ToSchema)]
pub struct SetMember {
  pub role: String,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct MemberPath {
  pub id: i64,
  pub user: i64,
}

/// Destructive project deletion requires the full namespace/project path.
#[derive(Deserialize, ToSchema)]
pub struct DeleteProject {
  pub confirmation: String,
}
