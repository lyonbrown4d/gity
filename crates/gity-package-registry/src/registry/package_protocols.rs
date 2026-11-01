use super::package_storage::{Upload, upload};
use super::packages::coordinate;
use super::*;
use crate::openapi::{package_schemas as spec, schemas as api};
use axum::{
  body::Body,
  extract::{OriginalUri, Path, State},
  http::HeaderMap,
  response::{IntoResponse, Response},
};
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared =
    utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(publish, download));
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(maven_publish, maven_download))
    .routes(utoipa_axum::routes!(
      super::package_metadata::npm,
      super::package_metadata::npm_publish
    ))
    .merge(super::package_npm_tags::documented_routes())
    .routes(utoipa_axum::routes!(super::package_metadata::nuget))
    .nest("/projects/{id}/packages/generic", shared.clone())
    .nest("/projects/{id}/packages/nuget", shared.clone())
    .nest("/projects/{id}/packages/pypi", shared)
    .merge(super::package_nuget::documented_routes())
    .merge(super::package_go::documented_routes())
    .merge(super::package_cargo::documented_routes())
    .merge(super::package_pypi::documented_routes())
    .merge(super::package_pypi_index::documented_routes())
    .layer(axum::middleware::from_fn(super::basic_challenge))
}
fn kind(uri: &axum::http::Uri) -> &str {
  uri
    .path()
    .split("/packages/")
    .nth(1)
    .and_then(|v| v.split('/').next())
    .unwrap_or("generic")
}
fn ctype(h: &HeaderMap) -> &str {
  h.get("content-type")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("application/octet-stream")
}
#[utoipa::path(
    put,path="/{name}/{version}/{*file}",tag="package protocols",
    params(api::ProtocolFilePath),
    request_body(content=api::Binary,content_type="application/octet-stream"),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageFile>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn publish(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, name, version, file)): Path<(i64, String, String, String)>,
  OriginalUri(uri): OriginalUri,
  body: Body,
) -> Reply {
  let actor = authorize_package_write(&s, &h, p).await?;
  reply(
    upload(
      &s,
      p,
      Upload {
        kind: kind(&uri),
        name: &name,
        version: &version,
        path: &file,
        content_type: ctype(&h),
        created_by_user_id: actor.user_id,
        source: &actor.source,
        pipeline_id: actor.pipeline_id,
        job_id: actor.job_id,
        registry_metadata: None,
      },
      body,
    )
    .await?,
  )
}
#[utoipa::path(
    get,path="/{name}/{version}/{*file}",tag="package protocols",
    params(api::ProtocolFilePath,("Range"=Option<String>,Header),("If-Range"=Option<String>,Header),("If-None-Match"=Option<String>,Header),("If-Match"=Option<String>,Header),("If-Modified-Since"=Option<String>,Header),("If-Unmodified-Since"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),(status=206,description="Requested single byte range",body=api::Binary,content_type="application/octet-stream"),(status=304,description="Cached representation is current"),(status=412,description="Conditional request precondition failed"),(status=416,description="Byte range is unsatisfiable"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, name, version, file)): Path<(i64, String, String, String)>,
  OriginalUri(uri): OriginalUri,
) -> Result<Response, AppError> {
  authorize(&s, &h, p, PackageAction::Read).await?;
  super::package_binary::response(
    &s,
    &h,
    coordinate(&s, p, kind(&uri), &name, &version, &file).await?,
  )
  .await
}
fn maven(file: &str) -> Result<(String, String), AppError> {
  let parts = file.split('/').collect::<Vec<_>>();
  if parts.len() < 3 {
    return Err(AppError::bad_request("maven package path is invalid"));
  }
  let artifact = parts[parts.len() - 3];
  let version = parts[parts.len() - 2];
  let group = parts[..parts.len() - 3].join(".");
  Ok((
    if group.is_empty() {
      artifact.to_owned()
    } else {
      format!("{group}:{artifact}")
    },
    version.into(),
  ))
}
#[utoipa::path(
    put,path="/projects/{id}/packages/maven/{*file}",tag="package protocols",
    params(api::MavenPath),
    request_body(content=api::Binary,content_type="application/octet-stream"),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageFile>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn maven_publish(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, file)): Path<(i64, String)>,
  body: Body,
) -> Result<Response, AppError> {
  let actor = authorize_package_write(&s, &h, p).await?;
  if let Some((_, checksum_type)) = super::package_maven::metadata_checksum_path(&file) {
    let checksum = axum::body::to_bytes(body, 1024)
      .await
      .map_err(|_| AppError::bad_request("invalid Maven metadata checksum"))?;
    super::package_maven::validate_metadata_checksum(&checksum, checksum_type)?;
    return Ok(axum::http::StatusCode::OK.into_response());
  }
  if file.ends_with("/maven-metadata.xml") {
    let metadata = axum::body::to_bytes(body, 2 * 1024 * 1024)
      .await
      .map_err(|_| AppError::bad_request("invalid Maven metadata request"))?;
    super::package_maven::validate_metadata_document(&metadata)?;
    return Ok(axum::http::StatusCode::OK.into_response());
  }
  let (name, version) = maven(&file)?;
  Ok(
    reply(
      upload(
        &s,
        p,
        Upload {
          kind: "maven",
          name: &name,
          version: &version,
          path: &file,
          content_type: ctype(&h),
          created_by_user_id: actor.user_id,
          source: &actor.source,
          pipeline_id: actor.pipeline_id,
          job_id: actor.job_id,
          registry_metadata: None,
        },
        body,
      )
      .await?,
    )?
    .into_response(),
  )
}
#[utoipa::path(
    get,path="/projects/{id}/packages/maven/{*file}",tag="package protocols",
    params(api::MavenPath,("Range"=Option<String>,Header),("If-Range"=Option<String>,Header),("If-None-Match"=Option<String>,Header),("If-Match"=Option<String>,Header),("If-Modified-Since"=Option<String>,Header),("If-Unmodified-Since"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),(status=206,description="Requested single byte range",body=api::Binary,content_type="application/octet-stream"),(status=304,description="Cached representation is current"),(status=412,description="Conditional request precondition failed"),(status=416,description="Byte range is unsatisfiable"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn maven_download(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, file)): Path<(i64, String)>,
) -> Result<Response, AppError> {
  authorize(&s, &h, p, PackageAction::Read).await?;
  if super::package_maven::metadata_checksum_path(&file).is_some() {
    return super::package_maven::metadata_checksum(&s, &h, p, &file).await;
  }
  if file.ends_with("/maven-metadata.xml") {
    return super::package_maven::metadata(&s, &h, p, &file).await;
  }
  let (name, version) = maven(&file)?;
  super::package_binary::response(
    &s,
    &h,
    coordinate(&s, p, "maven", &name, &version, &file).await?,
  )
  .await
}
