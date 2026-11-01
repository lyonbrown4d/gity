use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub is_super_admin: bool,
    pub status: UserStatus,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Disabled,
}
impl UserStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}
#[derive(Serialize, ToSchema)]
pub struct UserPage {
    pub items: Vec<User>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}
#[derive(Deserialize, utoipa::IntoParams)]
pub struct UserQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateUser {
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_super_admin: Option<bool>,
    #[schema(write_only, min_length = 8, max_length = 1024)]
    pub password: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateUser {
    pub status: Option<UserStatus>,
    /// A nonblank value replaces the password and revokes access/refresh sessions.
    #[schema(write_only, max_length = 1024)]
    pub password: Option<String>,
    #[schema(min_length = 1, max_length = 64, pattern = "^[A-Za-z0-9_.-]+$")]
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_super_admin: Option<bool>,
}
#[derive(Serialize, ToSchema)]
pub struct UserToken {
    pub id: String,
    pub user_id: String,
    pub name: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct CreatedUserToken {
    #[serde(flatten)]
    pub metadata: UserToken,
    pub token: String,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateUserToken {
    pub name: String,
}
#[derive(Serialize, ToSchema)]
pub struct OkResult {
    pub ok: bool,
}
#[derive(Serialize, ToSchema)]
pub struct Session {
    pub user_id: String,
    pub username: String,
    pub organization_id: Option<String>,
    pub token: String,
    pub refresh_token: String,
}
#[derive(Deserialize, ToSchema)]
pub struct Logout {
    #[schema(write_only)]
    pub refresh_token: Option<String>,
}
