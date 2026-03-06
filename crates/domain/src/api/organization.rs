use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Default, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct ListOrganizationsQuery {
  pub page: Option<u64>,
  pub page_size: Option<u64>,
  pub ids: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OrganizationView {
  pub id: String,
  pub key: String,
  pub name: String,
  pub role: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateOrganizationRequest {
  pub key: String,
  pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateOrganizationRequest {
  pub key: Option<String>,
  pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AddOrganizationMemberRequest {
  pub user_id: String,
  pub role: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OrganizationMemberView {
  pub organization_id: String,
  pub user_id: String,
  pub role: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OrganizationMemberDetailView {
  pub organization_id: String,
  pub user_id: String,
  pub username: String,
  pub email: String,
  pub role: String,
}
