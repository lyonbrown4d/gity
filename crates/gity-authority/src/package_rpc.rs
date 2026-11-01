use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use gity_core::AppError;
use gity_git_wire::identity_wire::{AuthorizePackageRequest, AuthorizePackageResponse};
use gity_package_registry::{PackageAction, PackageAuthorization};
use tonic::{Response, Status};

pub(crate) const CREDENTIAL_HEADERS: [&str; 4] = [
  "authorization",
  "private-token",
  "deploy-token",
  "job-token",
];
pub(crate) async fn authorize(
  delegate: Option<&dyn PackageAuthorization>,
  request: AuthorizePackageRequest,
) -> Result<Response<AuthorizePackageResponse>, Status> {
  let delegate = delegate.ok_or_else(|| Status::unavailable("package authority is disabled"))?;
  if request.project_id <= 0 || request.credentials.len() > CREDENTIAL_HEADERS.len() {
    return Err(Status::invalid_argument(
      "invalid package authorization context",
    ));
  }
  let action = match request.action.as_str() {
    "read" => PackageAction::Read,
    "write" => PackageAction::Write,
    "delete" => PackageAction::Delete,
    _ => return Err(Status::invalid_argument("invalid package action")),
  };
  let mut headers = HeaderMap::new();
  let mut size = 0;
  for (name, value) in request.credentials {
    size += value.len();
    if !CREDENTIAL_HEADERS.contains(&name.as_str()) || size > 16384 {
      return Err(Status::invalid_argument("invalid package credentials"));
    }
    headers.insert(
      HeaderName::from_bytes(name.as_bytes())
        .map_err(|_| Status::invalid_argument("invalid credential header"))?,
      HeaderValue::from_str(&value)
        .map_err(|_| Status::invalid_argument("invalid credential header"))?,
    );
  }
  let actor = delegate
    .authorize(request.project_id, action, &headers)
    .await
    .map_err(map_error)?;
  let actor_json = serde_json::to_vec(&actor)
    .map_err(|_| Status::internal("could not encode package decision"))?;
  Ok(Response::new(AuthorizePackageResponse { actor_json }))
}
fn map_error(error: AppError) -> Status {
  match error.status {
    StatusCode::UNAUTHORIZED => Status::unauthenticated("package credential rejected"),
    StatusCode::FORBIDDEN => Status::permission_denied("package operation forbidden"),
    StatusCode::NOT_FOUND => Status::not_found("package project not found"),
    StatusCode::BAD_REQUEST => Status::invalid_argument("invalid package authorization request"),
    _ => Status::unavailable("package authority is unavailable"),
  }
}
