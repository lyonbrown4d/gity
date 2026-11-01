use super::package_storage::{Upload, upload};
use super::*;
use crate::openapi::schemas as api;
use axum::{
  body::Body,
  extract::{Multipart, Path, State},
  http::HeaderMap,
  response::{IntoResponse, Response},
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(publish))
}

#[utoipa::path(
    post,path="/projects/{id}/packages/pypi/legacy/",tag="package protocols",
    params(api::ProjectPath),
    request_body(content_type="multipart/form-data"),
    responses((status=200,description="PyPI upload accepted"),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn publish(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(project_id): Path<i64>,
  mut multipart: Multipart,
) -> Result<Response, AppError> {
  let actor = authorize_package_write(&s, &h, project_id).await?;
  let mut name = None;
  let mut version = None;
  let mut filename = None;
  let mut package = None;
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_| AppError::bad_request("invalid PyPI upload multipart"))?
  {
    let field_name = field.name().unwrap_or_default().to_owned();
    if matches!(field_name.as_str(), "file" | "content") {
      filename = field.file_name().map(str::to_owned);
      package = Some(
        field
          .bytes()
          .await
          .map_err(|_| AppError::bad_request("failed to read Python package"))?,
      );
    } else {
      let value = field
        .text()
        .await
        .map_err(|_| AppError::bad_request("invalid PyPI metadata field"))?;
      match field_name.as_str() {
        "name" => name = Some(value),
        "version" => version = Some(value),
        _ => {}
      }
    }
  }
  let filename =
    filename.ok_or_else(|| AppError::bad_request("Python package filename is required"))?;
  let bytes = package.ok_or_else(|| AppError::bad_request("Python package file is required"))?;
  let metadata = gity_artifacts::python_metadata(&filename, &bytes)?;
  if name.as_deref().is_some_and(|value| {
    gity_artifacts::PackageFormat::Pypi
      .normalize_name(value)
      .ok()
      .as_deref()
      != Some(&metadata.name)
  }) || version
    .as_deref()
    .is_some_and(|value| value != metadata.version)
  {
    return Err(AppError::bad_request(
      "PyPI form metadata does not match the package file",
    ));
  }
  upload(
    &s,
    project_id,
    Upload {
      kind: "pypi",
      name: &metadata.name,
      version: &metadata.version,
      path: &filename,
      content_type: "application/octet-stream",
      created_by_user_id: actor.user_id,
      source: &actor.source,
      pipeline_id: actor.pipeline_id,
      job_id: actor.job_id,
      registry_metadata: Some(
        json!({
            "name":metadata.name,
            "version":metadata.version,
            "requires_python":metadata.requires_python,
            "requires_dist":metadata.requires_dist
        })
        .to_string(),
      ),
    },
    Body::from(bytes),
  )
  .await?;
  Ok((axum::http::StatusCode::OK, "OK\n").into_response())
}
