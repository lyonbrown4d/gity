use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct ProjectToken {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub name: String,
    pub username: String,
    pub scopes: Vec<String>,
    pub created_by_user_id: String,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
    pub active: bool,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct CreatedToken {
    pub project_token: ProjectToken,
    /// Raw token is returned only at creation.
    pub token: String,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateToken {
    pub name: String,
    pub username: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub created_by_user_id: Option<i64>,
}
#[derive(Serialize, ToSchema)]
pub struct DeployKey {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub fingerprint: String,
    pub public_key: String,
    pub can_push: bool,
    pub created_by_user_id: String,
    pub last_used_at: Option<String>,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateDeployKey {
    pub title: String,
    pub public_key: String,
    pub can_push: Option<bool>,
    pub created_by_user_id: Option<i64>,
}
#[derive(Serialize, ToSchema)]
pub struct Variable {
    pub id: String,
    pub project_id: String,
    pub key: String,
    /// Empty for masked variables.
    pub value: String,
    pub masked: bool,
    pub protected: bool,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct SetVariable {
    pub key: String,
    pub value: String,
    pub masked: Option<bool>,
    pub protected: Option<bool>,
}
#[derive(Serialize, ToSchema)]
pub struct AuditEvent {
    pub id: String,
    pub project_id: String,
    pub organization_id: String,
    pub event_name: String,
    pub action: String,
    pub actor_user_id: String,
    pub target_type: String,
    pub target_id: String,
    pub summary: String,
    pub payload: String,
    pub created_at: String,
}
