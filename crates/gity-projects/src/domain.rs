//! Commands describe business changes without HTTP or persistence dependencies.
#[derive(Clone, Debug)]
pub struct CreateProject {
  pub organization_id: i64,
  pub name: String,
  pub path_key: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
}
#[derive(Clone, Debug, Default)]
pub struct ProjectChanges {
  pub name: Option<String>,
  pub description: Option<Option<String>>,
  pub visibility: Option<String>,
  pub default_branch: Option<String>,
}
#[derive(Clone, Debug)]
pub struct CreateOrganization {
  pub name: String,
  pub path_key: String,
  pub description: Option<String>,
  pub visibility: String,
}
#[derive(Clone, Debug, Default)]
pub struct OrganizationChanges {
  pub name: Option<String>,
  pub path_key: Option<String>,
  pub description: Option<Option<String>>,
  pub visibility: Option<String>,
}
#[derive(Clone, Copy, Debug)]
pub enum OrganizationMemberChange<'a> {
  Add(&'a str),
  Set(&'a str),
  Remove,
}
#[derive(Clone, Debug)]
pub struct NewProjectToken {
  pub project_id: i64,
  pub kind: String,
  pub name: String,
  pub username: String,
  pub token_digest: String,
  pub scopes: String,
  pub created_by_user_id: i64,
  pub expires_at: Option<chrono::NaiveDateTime>,
}
#[derive(Clone, Debug)]
pub struct VariableCommand {
  pub key: String,
  pub value: String,
  pub masked: bool,
  pub protected: bool,
}
#[derive(Clone, Debug)]
pub struct ProtectionCommand {
  pub rule_type: String,
  pub push_access_level: String,
  pub merge_access_level: String,
  pub require_merge_request: bool,
  pub require_pipeline_success: bool,
  pub allow_force_push: bool,
  pub allow_delete: bool,
  pub actor: Option<i64>,
}
#[derive(Clone, Debug)]
pub struct LockQuery {
  pub cursor: i64,
  pub limit: u64,
  pub path: Option<String>,
  pub id: Option<i64>,
}
