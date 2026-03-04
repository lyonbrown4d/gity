use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// the input to our `create_user` handler
#[derive(Deserialize, ToSchema, Serialize)]
pub struct CreateUser {
  pub username: String,
  pub email: String,
  pub password: String,
}

// the output to our `create_user` handler
#[derive(Serialize, ToSchema, Deserialize)]
pub struct UserViewObject {
  pub id: String,
  pub username: String,
}

#[derive(Debug, ToSchema, Deserialize)]
pub struct UserQuery {
  pub page: Option<u64>,
  pub page_size: Option<u64>,
  pub username: Option<String>,
}
