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
  #[response(status = 401, description = "Authentication is missing or invalid")]
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
    description = "Database or object storage operation failed"
  )]
  Internal(ErrorEnvelope),
  #[response(status = 503, description = "Identity authority is unavailable")]
  AuthorityUnavailable(ErrorEnvelope),
}

#[derive(Serialize, ToSchema)]
pub struct Timestamps {
  #[schema(format=DateTime)]
  pub created_at: String,
  #[schema(format=DateTime)]
  pub updated_at: String,
}

#[derive(ToSchema)]
pub struct Binary(pub String);

#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ProjectPath {
  pub id: i64,
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

#[derive(Deserialize, ToSchema)]
pub struct CreatePackageFile {
  pub r#type: String,
  pub name: String,
  pub version: String,
  pub file_name: String,
  pub file_path: Option<String>,
  pub content_type: Option<String>,
  pub content_base64: String,
}
