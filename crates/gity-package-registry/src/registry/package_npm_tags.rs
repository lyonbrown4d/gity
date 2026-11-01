use super::packages::named;
use super::*;
use crate::openapi::schemas as api;
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(set_tag))
    .routes(utoipa_axum::routes!(delete_tag))
}

#[utoipa::path(
    put,
    path="/projects/{id}/packages/npm/-/package/{name}/dist-tags/{tag}",
    tag="package protocols",
    params(api::ProjectPath, api::PackageNamePath, api::ProtocolFilePath),
    request_body(content=String),
    responses((status=200,description="npm distribution tag updated"),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn set_tag(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path((project_id, package, tag)): Path<(i64, String, String)>,
  Json(version): Json<String>,
) -> Result<Json<Value>, AppError> {
  authorize_package_write(&state, &headers, project_id).await?;
  validate_tag(&tag)?;
  let package = named(&state, project_id, "npm", &package).await?;
  let exists = package["versions"]
    .as_array()
    .into_iter()
    .flatten()
    .any(|entry| text(&entry["version"], "version") == version);
  if !exists {
    return Err(AppError::not_found("npm package version not found"));
  }
  state
    .artifacts
    .repository
    .set_npm_dist_tag(
      project_id,
      text(&package["package"], "name"),
      &tag,
      Some(&version),
    )
    .await?;
  Ok(Json(json!(version)))
}

#[utoipa::path(
    delete,
    path="/projects/{id}/packages/npm/-/package/{name}/dist-tags/{tag}",
    tag="package protocols",
    params(api::ProjectPath, api::PackageNamePath, api::ProtocolFilePath),
    responses((status=200,description="npm distribution tag deleted"),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn delete_tag(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path((project_id, package, tag)): Path<(i64, String, String)>,
) -> Result<Json<Value>, AppError> {
  let actor = authorize_package_write(&state, &headers, project_id).await?;
  if actor.source == "ci_job" {
    return Err(AppError::forbidden(
      "CI job tokens cannot delete registry metadata",
    ));
  }
  validate_tag(&tag)?;
  let package = named(&state, project_id, "npm", &package).await?;
  state
    .artifacts
    .repository
    .set_npm_dist_tag(project_id, text(&package["package"], "name"), &tag, None)
    .await?;
  Ok(Json(json!({"ok":true})))
}

fn validate_tag(tag: &str) -> Result<(), AppError> {
  if tag.is_empty()
    || tag.len() > 214
    || tag.chars().any(char::is_whitespace)
    || semver::Version::parse(tag).is_ok()
  {
    return Err(AppError::bad_request("invalid npm dist-tag"));
  }
  Ok(())
}
