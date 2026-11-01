use crate::api;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrganizationRow {
  pub id: String,
  pub name: String,
  pub key: String,
  pub role: String,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MemberRow {
  pub user_id: String,
  pub username: String,
  pub email: String,
  pub role: String,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectRow {
  pub id: String,
  pub organization_id: String,
  pub name: String,
  pub key: String,
  pub full_path: String,
  pub default_branch: String,
  pub description: String,
  pub visibility: String,
  pub status: String,
  pub clone_http_url: String,
  pub uuid: String,
}
pub(crate) fn organizations(value: &Value) -> Vec<OrganizationRow> {
  api::list(value)
    .iter()
    .map(|row| OrganizationRow {
      id: api::text(row, "id"),
      name: api::text(row, "name"),
      key: api::text(row, "key"),
      role: api::text(row, "role"),
    })
    .collect()
}
pub(crate) fn members(value: &Value) -> Vec<MemberRow> {
  api::list(value)
    .iter()
    .map(|row| MemberRow {
      user_id: api::text(row, "user_id"),
      username: api::text(row, "username"),
      email: api::text(row, "email"),
      role: api::text(row, "role"),
    })
    .collect()
}
pub(crate) fn projects(value: &Value) -> Vec<ProjectRow> {
  api::list(value)
    .iter()
    .map(|row| ProjectRow {
      id: api::text(row, "id"),
      organization_id: api::text(row, "organization_id"),
      name: api::text(row, "name"),
      key: api::text(row, "key"),
      full_path: api::text(row, "full_path"),
      default_branch: api::text(row, "default_branch"),
      description: api::text(row, "description"),
      visibility: api::text(row, "visibility"),
      status: api::text(row, "status"),
      clone_http_url: api::text(row, "clone_http_url"),
      uuid: api::text(row, "uuid"),
    })
    .collect()
}
pub(crate) fn projects_in_organization(rows: &[ProjectRow], organization: &str) -> Vec<ProjectRow> {
  rows
    .iter()
    .filter(|row| row.organization_id == organization && !organization.is_empty())
    .cloned()
    .collect()
}
pub(crate) fn confirmation_matches(project: &ProjectRow, input: &str) -> bool {
  !project.full_path.is_empty() && input == project.full_path
}
#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
