use super::packages::{coordinate, named};
use super::*;
use crate::openapi::schemas as api;
use axum::{
  extract::{Path, State},
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::{IntoResponse, Response},
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(proxy))
}

#[utoipa::path(
    get,path="/projects/{id}/packages/go/{*module_and_version_path}",tag="package protocols",
    params(("id"=i64,Path),("module_and_version_path"=String,Path)),
    responses((status=200,description="Go module proxy response"),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn proxy(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project_id, module_and_version_path)): Path<(i64, String)>,
) -> Result<Response, AppError> {
  authorize(&s, &h, project_id, PackageAction::Read).await?;
  let (escaped_module, file) = module_and_version_path
    .split_once("/@v/")
    .ok_or_else(|| AppError::not_found("Go module version path not found"))?;
  let module = gity_artifacts::go_unescape_path(escaped_module)?;
  if file == "list" {
    let detail = named(&s, project_id, "go", &module).await?;
    let versions = listed_versions(&detail);
    let mut body = versions.join("\n");
    if !body.is_empty() {
      body.push('\n');
    }
    return Ok(
      (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
      )
        .into_response(),
    );
  }
  let (version, suffix) = [".info", ".mod", ".zip"]
    .iter()
    .find_map(|suffix| file.strip_suffix(suffix).map(|version| (version, *suffix)))
    .ok_or_else(|| AppError::not_found("unsupported Go module proxy request"))?;
  let version = decode_go_version(version)?;
  let name = module.as_str();
  if suffix == ".info" {
    let detail = named(&s, project_id, "go", name).await?;
    let version_row = detail["versions"]
      .as_array()
      .into_iter()
      .flatten()
      .find(|row| text(&row["version"], "version") == version)
      .ok_or_else(|| AppError::not_found("Go module version not found"))?;
    let created = text(&version_row["version"], "created_at");
    let time = chrono::DateTime::parse_from_rfc3339(created)
      .map(|value| value.to_utc())
      .or_else(|_| {
        chrono::NaiveDateTime::parse_from_str(created, "%Y-%m-%dT%H:%M:%S%.f")
          .map(|value| value.and_utc())
      })
      .map(|value| value.to_rfc3339())
      .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());
    return Ok(axum::Json(json!({"Version":version,"Time":time})).into_response());
  }
  let coordinate = coordinate(
    &s,
    project_id,
    "go",
    name,
    &version,
    &format!("{version}{suffix}"),
  )
  .await?;
  let mut response = super::package_binary::response(&s, &h, coordinate).await?;
  response.headers_mut().insert(
    header::CONTENT_TYPE,
    HeaderValue::from_static(if suffix == ".mod" {
      "text/plain; charset=utf-8"
    } else {
      "application/zip"
    }),
  );
  Ok(response)
}

fn decode_go_version(version: &str) -> Result<String, AppError> {
  gity_artifacts::go_unescape_path(version).map_err(Into::into)
}

fn listed_versions(detail: &Value) -> Vec<String> {
  let mut versions = detail["versions"]
    .as_array()
    .into_iter()
    .flatten()
    .filter(|version| text(&version["version"], "status") != "unlisted")
    .filter_map(|version| version["version"]["version"].as_str().map(str::to_owned))
    .collect::<Vec<_>>();
  versions.sort_by(|left, right| {
    semver::Version::parse(left.strip_prefix('v').unwrap_or(left))
      .ok()
      .cmp(&semver::Version::parse(right.strip_prefix('v').unwrap_or(right)).ok())
  });
  versions
}

#[cfg(test)]
#[path = "package_go_tests.rs"]
mod tests;
