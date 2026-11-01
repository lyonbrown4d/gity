use super::*;

pub(super) async fn named(
  state: &AppState,
  project_id: i64,
  kind: &str,
  name: &str,
) -> Result<Value, AppError> {
  let format = gity_artifacts::PackageFormat::parse(kind)
    .ok_or_else(|| AppError::bad_request("unsupported package format"))?;
  let name = format.normalize_name(name)?;
  let rows = state
    .artifacts
    .repository
    .all_versions(project_id, format, &name)
    .await?;
  if rows.is_empty() {
    return Err(AppError::not_found("package not found"));
  }
  let versions = rows
    .into_iter()
    .map(|(version, files)| {
      json!({
        "version": {
          "id":version.id,"project_package_id":version.project_package_id,
          "version":version.version,"status":version.status,
          "created_at":version.created_at,"updated_at":version.updated_at
        },
        "files":files.iter().map(super::package_storage::protocol_file_value).collect::<Vec<_>>()
      })
    })
    .collect::<Vec<_>>();
  Ok(json!({"package":{"project_id":project_id,"type":kind,"name":name},"versions":versions}))
}

pub(super) async fn file(
  state: &AppState,
  project_id: i64,
  file_id: i64,
) -> Result<Value, AppError> {
  state
    .artifacts
    .repository
    .find_file_by_id(project_id, file_id)
    .await?
    .map(|row| super::package_storage::file_value(&row))
    .ok_or_else(|| AppError::not_found("package file not found"))
}

pub(super) async fn coordinate(
  state: &AppState,
  project_id: i64,
  kind: &str,
  name: &str,
  version: &str,
  path: &str,
) -> Result<Value, AppError> {
  let format = gity_artifacts::PackageFormat::parse(kind)
    .ok_or_else(|| AppError::bad_request("unsupported package format"))?;
  let coordinate =
    gity_artifacts::ArtifactCoordinate::new(project_id, format, name, version, path)?;
  state
    .artifacts
    .repository
    .find_file(&coordinate)
    .await?
    .map(|row| super::package_storage::file_value(&row))
    .ok_or_else(|| AppError::not_found("package file not found"))
}
