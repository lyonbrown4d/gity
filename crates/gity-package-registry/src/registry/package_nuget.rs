use super::package_storage::{Upload, upload};
use super::packages::named;
use super::*;
use crate::openapi::schemas as api;
use axum::{
  body::Body,
  extract::{Multipart, Path, State},
  http::{HeaderMap, StatusCode},
  response::{IntoResponse, Response},
};
use std::collections::BTreeSet;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(versions))
    .routes(utoipa_axum::routes!(download))
    .routes(utoipa_axum::routes!(super::package_nuget_search::search))
    .routes(utoipa_axum::routes!(push))
    .route(
      "/projects/{id}/packages/nuget/push/",
      axum::routing::put(push),
    )
    .routes(utoipa_axum::routes!(unlist))
    .routes(utoipa_axum::routes!(relist))
    .merge(super::package_nuget_registration::documented_routes())
}

#[utoipa::path(
    get,path="/projects/{id}/packages/nuget/flatcontainer/{name}/index.json",tag="package protocols",
    params(api::ProjectPath,api::PackageNamePath),
    responses((status=200,description="Available normalized NuGet versions",body=String),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn versions(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, name)): Path<(i64, String)>,
) -> Result<Response, AppError> {
  authorize(&s, &h, project_id, PackageAction::Read).await?;
  let detail = named(&s, project_id, "nuget", &name).await?;
  let mut versions = package_versions(&detail);
  versions.sort_by(|left, right| {
    semver::Version::parse(left)
      .ok()
      .cmp(&semver::Version::parse(right).ok())
  });
  Ok(axum::Json(json!({"versions":versions})).into_response())
}

fn package_versions(detail: &Value) -> Vec<String> {
  detail["versions"]
    .as_array()
    .into_iter()
    .flatten()
    .filter_map(|version| version["version"]["version"].as_str())
    .map(str::to_ascii_lowercase)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect()
}

#[cfg(test)]
#[path = "package_nuget_tests.rs"]
mod tests;

#[utoipa::path(
    get,path="/projects/{id}/packages/nuget/flatcontainer/{name}/{version}/{file}",tag="package protocols",
    params(api::ProjectPath,api::ProtocolFilePath),
    responses((status=200,description="NuGet package content",body=api::Binary,content_type="application/octet-stream"),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, name, version, file)): Path<(i64, String, String, String)>,
) -> Result<Response, AppError> {
  authorize(&s, &h, project_id, PackageAction::Read).await?;
  let name = name.to_ascii_lowercase();
  let file = file.to_ascii_lowercase();
  super::package_binary::response(
    &s,
    &h,
    super::packages::coordinate(&s, project_id, "nuget", &name, &version, &file).await?,
  )
  .await
}

#[utoipa::path(
    put,path="/projects/{id}/packages/nuget/push",tag="package protocols",
    params(api::ProjectPath),
    request_body(content_type="multipart/form-data"),
    responses((status=201,description="NuGet package published"),api::Errors),
    security(("private_token"=[]),("basic"=[]))
)]
async fn push(
  State(s): State<AppState>,
  mut h: HeaderMap,
  Path(project_id): Path<i64>,
  mut multipart: Multipart,
) -> Result<Response, AppError> {
  if !h.contains_key("authorization")
    && !h.contains_key("private-token")
    && let Some(api_key) = h.get("x-nuget-apikey").cloned()
  {
    h.insert("private-token", api_key);
  }
  let actor = authorize_package_write(&s, &h, project_id).await?;
  let field = multipart
    .next_field()
    .await
    .map_err(|_| AppError::bad_request("invalid NuGet multipart upload"))?
    .ok_or_else(|| AppError::bad_request("NuGet package file is required"))?;
  let data = field
    .bytes()
    .await
    .map_err(|_| AppError::bad_request("failed to read NuGet package"))?;
  let metadata = gity_artifacts::nuget_metadata(&data)?;
  let name = metadata.id.to_ascii_lowercase();
  let version = metadata.version.to_ascii_lowercase();
  let file = format!("{name}.{version}.nupkg");
  let uploaded = upload(
    &s,
    project_id,
    Upload {
      kind: "nuget",
      name: &name,
      version: &version,
      path: &file,
      content_type: "application/octet-stream",
      created_by_user_id: actor.user_id,
      source: &actor.source,
      pipeline_id: actor.pipeline_id,
      job_id: actor.job_id,
      registry_metadata: Some(
        json!({
            "id":metadata.id,
            "version":metadata.version,
            "dependency_groups":metadata.dependency_groups.iter().map(|group| json!({
                "target_framework":group.target_framework,
                "dependencies":group.dependencies.iter().map(|dependency| json!({
                    "id":dependency.id,"range":dependency.range
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        })
        .to_string(),
      ),
    },
    Body::from(data),
  )
  .await?;
  Ok((StatusCode::CREATED, axum::Json(uploaded)).into_response())
}

#[utoipa::path(
    delete,path="/projects/{id}/packages/nuget/push/{name}/{version}",tag="package protocols",
    params(api::ProjectPath,api::PackageNamePath,("version"=String,Path)),
    responses((status=204,description="NuGet package unlisted"),api::Errors),
    security(("private_token"=[]),("basic"=[]))
)]
async fn unlist(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, name, version)): Path<(i64, String, String)>,
) -> Result<StatusCode, AppError> {
  authorize_package_delete(&s, &h, project_id).await?;
  let normalized = gity_artifacts::PackageFormat::Nuget.normalize_name(&name)?;
  s.artifacts
    .repository
    .set_version_status(
      project_id,
      gity_artifacts::PackageFormat::Nuget,
      &normalized,
      &version,
      "unlisted",
    )
    .await?;
  Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,path="/projects/{id}/packages/nuget/push/{name}/{version}",tag="package protocols",
    params(api::ProjectPath,api::PackageNamePath,("version"=String,Path)),
    responses((status=200,description="NuGet package relisted"),api::Errors),
    security(("private_token"=[]),("basic"=[]))
)]
async fn relist(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, name, version)): Path<(i64, String, String)>,
) -> Result<StatusCode, AppError> {
  authorize_package_delete(&s, &h, project_id).await?;
  let normalized = gity_artifacts::PackageFormat::Nuget.normalize_name(&name)?;
  s.artifacts
    .repository
    .set_version_status(
      project_id,
      gity_artifacts::PackageFormat::Nuget,
      &normalized,
      &version,
      "default",
    )
    .await?;
  Ok(StatusCode::OK)
}
