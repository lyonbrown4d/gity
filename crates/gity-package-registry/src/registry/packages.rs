pub(super) use super::package_lookup::{coordinate, file, named};
use super::package_storage::{Upload, upload};
use super::*;
use crate::openapi::{package_schemas as spec, schemas as api};
use axum::{
  body::Body,
  extract::{DefaultBodyLimit, Path, Query, State},
  http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::STANDARD};
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list))
    .routes(utoipa_axum::routes!(detail))
    .routes(utoipa_axum::routes!(upload_json))
    .routes(utoipa_axum::routes!(content))
    .routes(utoipa_axum::routes!(download))
    .routes(utoipa_axum::routes!(download_named))
    .routes(utoipa_axum::routes!(delete_package))
    .routes(utoipa_axum::routes!(delete_version))
    .routes(utoipa_axum::routes!(delete_file));
  utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared)
    .merge(super::package_protocols::documented_routes())
    .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
}
#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in=Query)]
struct PackageListQuery {
  #[param(nullable = true)]
  format: Option<String>,
  #[param(nullable = true)]
  name: Option<String>,
  #[param(minimum = 1)]
  page: Option<u64>,
  #[param(minimum = 1, maximum = 100)]
  per_page: Option<u64>,
}
#[utoipa::path(
    get,path="/{id}/packages",tag="packages",
    params(api::ProjectPath,PackageListQuery),
    responses((status=200,description="Paginated package list"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn list(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(query): Query<PackageListQuery>,
) -> Reply {
  authorize(&s, &h, p, PackageAction::Read).await?;
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
  let (packages, total) = s
    .artifacts
    .repository
    .list_packages(p, format, query.name.as_deref(), page, per_page)
    .await?;
  reply(json!({
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
#[utoipa::path(
    get,path="/{id}/packages/{package}",tag="packages",
    params(api::PackagePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageDetail>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, id)): Path<(i64, i64)>) -> Reply {
  authorize(&s, &h, p, PackageAction::Read).await?;
  reply(package(&s, p, id).await?)
}
pub(super) async fn package(s: &AppState, p: i64, id: i64) -> Result<Value, AppError> {
  let (package, details) = s
    .artifacts
    .repository
    .package_detail(p, id)
    .await?
    .ok_or_else(|| AppError::not_found("resource not found"))?;
  let versions = details
    .into_iter()
    .map(|(version, files)| {
      json!({
        "version": {
          "id":version.id,
          "project_package_id":version.project_package_id,
          "version":version.version,
          "status":version.status,
          "created_at":version.created_at,
          "updated_at":version.updated_at
        },
        "files":files.iter().map(super::package_storage::file_value).collect::<Vec<_>>()
      })
    })
    .collect::<Vec<_>>();
  Ok(json!({
    "package": {
      "id":package.id,
      "project_id":package.project_id,
      "type":package.r#type,
      "name":package.name,
      "created_at":package.created_at,
      "updated_at":package.updated_at
    },
    "versions":versions
  }))
}
#[utoipa::path(
    post,path="/{id}/packages/files",tag="packages",
    params(api::ProjectPath),
    request_body(content=spec::CreatePackageFile),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageFile>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn upload_json(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = authorize_package_write(&s, &h, p).await?;
  let data = STANDARD
    .decode(required(&b, "content_base64")?)
    .map_err(|_| AppError::bad_request("invalid content_base64"))?;
  let path = if text(&b, "file_path").is_empty() {
    required(&b, "file_name")?
  } else {
    text(&b, "file_path").to_owned()
  };
  reply(
    upload(
      &s,
      p,
      Upload {
        kind: &required(&b, "type")?,
        name: &required(&b, "name")?,
        version: &required(&b, "version")?,
        path: &path,
        content_type: text(&b, "content_type"),
        created_by_user_id: actor.user_id,
        source: &actor.source,
        pipeline_id: actor.pipeline_id,
        job_id: actor.job_id,
        registry_metadata: None,
      },
      Body::from(data),
    )
    .await?,
  )
}
#[utoipa::path(
    get,path="/{id}/packages/files/{file}",tag="packages",
    params(api::FilePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageContent>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn content(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id)): Path<(i64, i64)>,
) -> Reply {
  authorize(&s, &h, p, PackageAction::Read).await?;
  let row = file(&s, p, id).await?;
  let content = s.storage.get(text(&row, "storage_key")).await?;
  reply(json!({"file":row,"content_base64":STANDARD.encode(content)}))
}
#[utoipa::path(
    get,path="/{id}/packages/files/{file}/download",tag="packages",
    params(api::FilePath,("Range"=Option<String>,Header),("If-Range"=Option<String>,Header),("If-None-Match"=Option<String>,Header),("If-Match"=Option<String>,Header),("If-Modified-Since"=Option<String>,Header),("If-Unmodified-Since"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),(status=206,description="Requested single byte range",body=api::Binary,content_type="application/octet-stream"),(status=304,description="Cached representation is current"),(status=412,description="Conditional request precondition failed"),(status=416,description="Byte range is unsatisfiable"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
  authorize(&s, &h, p, PackageAction::Read).await?;
  super::package_binary::response(&s, &h, file(&s, p, id).await?).await
}

#[utoipa::path(
    get,path="/{id}/packages/files/{file}/download/{filename}",tag="packages",
    params(api::FilePath, ("filename"=String,Path), ("Range"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download_named(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id, _filename)): Path<(i64, i64, String)>,
) -> Result<axum::response::Response, AppError> {
  authorize(&s, &h, p, PackageAction::Read).await?;
  super::package_binary::response(&s, &h, file(&s, p, id).await?).await
}

#[utoipa::path(
    delete,path="/{id}/packages/{package}",tag="packages",
    params(api::PackagePath),
    responses((status=204,description="Package and all its versions were deleted"),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn delete_package(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, package_id)): Path<(i64, i64)>,
) -> Result<axum::http::StatusCode, AppError> {
  authorize_package_delete(&s, &h, project_id).await?;
  s.artifacts
    .repository
    .delete_package(project_id, package_id)
    .await?;
  let _ = s.artifacts.cleanup_pending(100).await;
  Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,path="/{id}/packages/{package}/versions/{version}",tag="packages",
    params(api::PackagePath,("version"=String,Path,description="Package version")),
    responses((status=204,description="Package version and its files were deleted"),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn delete_version(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, package_id, version)): Path<(i64, i64, String)>,
) -> Result<axum::http::StatusCode, AppError> {
  authorize_package_delete(&s, &h, project_id).await?;
  s.artifacts
    .repository
    .delete_version(project_id, package_id, &version)
    .await?;
  let _ = s.artifacts.cleanup_pending(100).await;
  Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,path="/{id}/packages/files/{file}",tag="packages",
    params(api::FilePath),
    responses((status=204,description="Package file was deleted"),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn delete_file(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, file_id)): Path<(i64, i64)>,
) -> Result<axum::http::StatusCode, AppError> {
  authorize_package_delete(&s, &h, project_id).await?;
  s.artifacts
    .repository
    .delete_file(project_id, file_id)
    .await?;
  let _ = s.artifacts.cleanup_pending(100).await;
  Ok(axum::http::StatusCode::NO_CONTENT)
}
