use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Default, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct ListUsersQuery {
  pub limit: Option<u64>,
  pub ids: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UserView {
  pub id: String,
  pub username: String,
  pub email: String,
  pub status: String,
  pub is_super_admin: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateMeRequest {
  pub username: Option<String>,
  pub email: Option<String>,
  pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
  pub username: String,
  pub email: String,
  pub password: String,
}
