use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, IntoResponses, ToSchema};
#[derive(Serialize, ToSchema)]
pub struct Envelope<T> {
  pub code: i32,
  pub message: String,
  pub data: T,
}
#[derive(Serialize, ToSchema)]
pub struct ErrorEnvelope {
  pub code: u16,
  pub message: String,
  pub data: Option<serde_json::Value>,
}
#[derive(IntoResponses)]
pub enum Errors {
  #[response(
    status = 400,
    description = "Invalid input or domain validation failure"
  )]
  Invalid(ErrorEnvelope),
  #[response(
    status = 401,
    description = "Authentication is missing, invalid, expired or revoked"
  )]
  Unauthenticated(ErrorEnvelope),
  #[response(
    status = 403,
    description = "Project role or token scope does not permit this operation"
  )]
  Forbidden(ErrorEnvelope),
  #[response(
    status = 404,
    description = "Resource does not exist in the authorized project"
  )]
  Missing(ErrorEnvelope),
  #[response(
    status = 409,
    description = "Conflicting resource or invalid state transition"
  )]
  Conflict(ErrorEnvelope),
  #[response(status = 413, description = "Request body exceeds the upload limit")]
  TooLarge(ErrorEnvelope),
  #[response(status = 422, description = "Request JSON cannot be decoded")]
  Unprocessable(ErrorEnvelope),
  #[response(
    status = 500,
    description = "Database, Git or object storage operation failed"
  )]
  Internal(ErrorEnvelope),
  #[response(
    status = 503,
    description = "A required component is unavailable or awaiting recovery"
  )]
  Unavailable(ErrorEnvelope),
}
#[derive(Serialize, ToSchema)]
pub struct Status {
  pub status: String,
}
#[derive(Serialize, ToSchema)]
pub struct Deleted {
  pub deleted: bool,
}
#[derive(Serialize, ToSchema)]
pub struct Timestamps {
  #[schema(format=DateTime)]
  pub created_at: String,
  #[schema(format=DateTime)]
  pub updated_at: String,
}
#[derive(Deserialize, ToSchema)]
pub struct UserIds {
  pub user_ids: Vec<i64>,
}
/// Shared list filters for issue and merge request collections.
#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
  /// `open`/`closed` for issues, `opened`/`merged`/`closed` for merge requests; `all` disables the filter.
  pub state: Option<String>,
  /// 1-based page number; defaults to 1.
  pub page: Option<u64>,
  /// Items per page, 1-100; defaults to 20.
  pub page_size: Option<u64>,
}

#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ProjectPath {
  pub id: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct IidPath {
  pub id: i64,
  pub iid: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct AttachmentPath {
  pub id: i64,
  pub iid: i64,
  pub attachment: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct WikiPath {
  pub id: i64,
  pub slug: String,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct RulePath {
  pub id: i64,
  pub rule: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ReleasePath {
  pub id: i64,
  pub release: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct LinkPath {
  pub id: i64,
  pub release: i64,
  pub link: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct TokenPath {
  pub id: i64,
  pub token: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct KeyPath {
  pub id: i64,
  pub key: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct VariablePath {
  pub id: i64,
  pub key: String,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct LockPath {
  pub id: i64,
  pub lock: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct PackagePath {
  pub id: i64,
  pub package: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct FilePath {
  pub id: i64,
  pub file: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ProtocolFilePath {
  pub id: i64,
  pub name: String,
  pub version: String,
  pub file: String,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct MavenPath {
  pub id: i64,
  pub file: String,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct PackageNamePath {
  pub id: i64,
  pub name: String,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct TodoPath {
  pub id: i64,
}
#[derive(ToSchema)]
#[schema(value_type=String,format=Binary)]
pub struct Binary(pub String);
