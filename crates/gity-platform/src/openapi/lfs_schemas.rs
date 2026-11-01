use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct LfsObject {
  pub id: String,
  pub project_id: String,
  pub oid: String,
  pub byte_size: i64,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct LfsObjects {
  pub objects: Vec<LfsObject>,
  pub next_cursor: String,
}
#[derive(Serialize, ToSchema)]
pub struct LockOwner {
  pub name: String,
}
#[derive(Serialize, ToSchema)]
pub struct LfsLock {
  pub id: String,
  pub path: String,
  pub locked_at: String,
  pub owner: LockOwner,
}
#[derive(Serialize, ToSchema)]
pub struct LockEnvelope {
  pub lock: LfsLock,
}
#[derive(Serialize, ToSchema)]
pub struct LfsLocks {
  pub locks: Vec<LfsLock>,
  pub next_cursor: String,
}
#[derive(Serialize, ToSchema)]
pub struct VerifiedLocks {
  pub ours: Vec<LfsLock>,
  pub theirs: Vec<LfsLock>,
  pub next_cursor: String,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateLock {
  pub path: String,
}
#[derive(Deserialize, ToSchema)]
pub struct Unlock {
  pub force: Option<bool>,
}
#[derive(Deserialize, ToSchema)]
pub struct BatchObject {
  pub oid: String,
  pub size: i64,
}
#[derive(Deserialize, ToSchema)]
pub struct BatchRequest {
  pub operation: String,
  pub transfers: Option<Vec<String>>,
  pub objects: Vec<BatchObject>,
}
#[derive(Serialize, ToSchema)]
pub struct ActionError {
  pub code: u16,
  pub message: String,
}
#[derive(Serialize, ToSchema)]
pub struct LfsAction {
  pub href: String,
  pub header: Option<std::collections::BTreeMap<String, String>>,
}
#[derive(Serialize, ToSchema)]
pub struct BatchObjectResult {
  pub oid: String,
  pub size: i64,
  pub actions: Option<std::collections::BTreeMap<String, LfsAction>>,
  pub error: Option<ActionError>,
}
#[derive(Serialize, ToSchema)]
pub struct BatchResponse {
  pub transfer: String,
  pub objects: Vec<BatchObjectResult>,
}
#[derive(Deserialize, ToSchema)]
pub struct VerifyLocks {
  pub cursor: Option<String>,
  pub limit: Option<i64>,
}
#[derive(utoipa::IntoParams)]
#[into_params(parameter_in=Path)]
pub struct RepositoryPath {
  pub organization: String,
  /// Repository name including the .git suffix.
  pub project: String,
}
#[derive(utoipa::IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ObjectPath {
  pub organization: String,
  pub project: String,
  pub oid: String,
}
#[derive(utoipa::IntoParams)]
#[into_params(parameter_in=Path)]
pub struct UnlockPath {
  pub organization: String,
  pub project: String,
  pub lock: i64,
}
