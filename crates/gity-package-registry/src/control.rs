//! Typed control operations shared by REST and platform application adapters.
use crate::{PackageAction, PackageState};
use axum::http::HeaderMap;
use gity_core::AppError;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PackageListQuery {
  #[param(nullable = true)]
  pub format: Option<String>,
  #[param(nullable = true)]
  pub name: Option<String>,
  #[param(minimum = 1)]
  pub page: Option<u64>,
  #[param(minimum = 1, maximum = 100)]
  pub per_page: Option<u64>,
}

pub async fn list(
  state: &PackageState,
  headers: &HeaderMap,
  project_id: i64,
  query: PackageListQuery,
) -> Result<Value, AppError> {
  crate::registry::authorize(state, headers, project_id, PackageAction::Read).await?;
  let format = query
    .format
    .as_deref()
    .map(|value| {
      gity_artifacts::PackageFormat::parse(value)
        .ok_or_else(|| AppError::bad_request("unsupported package format"))
    })
    .transpose()?;
  let page = query.page.unwrap_or(1).max(1);
  let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
  let (packages, total) = state
    .artifacts
    .repository
    .list_packages(project_id, format, query.name.as_deref(), page, per_page)
    .await?;
  Ok(json!({
    "items": packages.into_iter().map(|package| json!({
      "id": package.id,
      "project_id": package.project_id,
      "type": package.r#type,
      "name": package.name,
      "created_at": package.created_at,
      "updated_at": package.updated_at,
    })).collect::<Vec<_>>(),
    "page": page,
    "per_page": per_page,
    "total": total,
    "total_pages": total.div_ceil(per_page),
  }))
}

pub async fn detail(
  state: &PackageState,
  headers: &HeaderMap,
  project_id: i64,
  package_id: i64,
) -> Result<Value, AppError> {
  crate::registry::authorize(state, headers, project_id, PackageAction::Read).await?;
  crate::registry::packages::package(state, project_id, package_id).await
}

pub async fn delete_package(
  state: &PackageState,
  headers: &HeaderMap,
  project_id: i64,
  package_id: i64,
) -> Result<(), AppError> {
  crate::registry::authorize_package_delete(state, headers, project_id).await?;
  state
    .artifacts
    .repository
    .delete_package(project_id, package_id)
    .await?;
  let _ = state.artifacts.cleanup_pending(100).await;
  Ok(())
}

pub async fn delete_version(
  state: &PackageState,
  headers: &HeaderMap,
  project_id: i64,
  package_id: i64,
  version: &str,
) -> Result<(), AppError> {
  crate::registry::authorize_package_delete(state, headers, project_id).await?;
  state
    .artifacts
    .repository
    .delete_version(project_id, package_id, version)
    .await?;
  let _ = state.artifacts.cleanup_pending(100).await;
  Ok(())
}

pub async fn delete_file(
  state: &PackageState,
  headers: &HeaderMap,
  project_id: i64,
  file_id: i64,
) -> Result<(), AppError> {
  crate::registry::authorize_package_delete(state, headers, project_id).await?;
  state
    .artifacts
    .repository
    .delete_file(project_id, file_id)
    .await?;
  let _ = state.artifacts.cleanup_pending(100).await;
  Ok(())
}
