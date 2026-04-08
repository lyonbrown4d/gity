use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamespaceCommand {
  pub parent_namespace_id: Option<i64>,
  pub owner_user_id: Option<String>,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub kind: String,
  pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceView {
  pub id: i64,
  pub full_path: String,
  pub parent_namespace_id: Option<i64>,
  pub owner_user_id: Option<String>,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub kind: String,
  pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNamespaceCommand {
  pub namespace_id: i64,
  pub path_key: Option<String>,
  pub name: Option<String>,
  pub description: Option<Option<String>>,
  pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceMemberView {
  pub id: i64,
  pub namespace_id: i64,
  pub user_id: String,
  pub role: String,
  pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInvitationView {
  pub id: i64,
  pub namespace_id: i64,
  pub email: String,
  pub role: String,
  pub state: String,
  pub invited_by_user_id: String,
  pub accepted_by_user_id: Option<String>,
  pub expires_at_unix: Option<i64>,
}
