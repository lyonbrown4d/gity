use super::packages::named;
use super::*;
use crate::openapi::schemas as api;
use axum::{
  extract::{Path, State},
  http::HeaderMap,
  response::{Html, IntoResponse, Response},
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(simple_index))
    .routes(utoipa_axum::routes!(simple_package))
}

#[utoipa::path(
    get,path="/projects/{id}/packages/pypi/simple",tag="package protocols",
    params(api::ProjectPath),
    responses((status=200,description="Python Simple repository index",body=String),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn simple_index(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(project_id): Path<i64>,
) -> Result<Response, AppError> {
  authorize(&state, &headers, project_id, PackageAction::Read).await?;
  let (packages, _) = state
    .artifacts
    .repository
    .list_packages(
      project_id,
      Some(gity_artifacts::PackageFormat::Pypi),
      None,
      1,
      100_000,
    )
    .await?;
  if accepts_json(&headers) {
    return Ok((
            [("content-type", "application/vnd.pypi.simple.v1+json")],
            axum::Json(json!({"meta":{"api-version":"1.0"},"projects":packages.iter().map(|p| json!({"name":p.name})).collect::<Vec<_>>()})),
        ).into_response());
  }
  let mut html = String::from("<!doctype html><html><body>\n");
  for package in packages {
    let name = escape(package.name);
    html.push_str(&format!("<a href=\"./{name}/\">{name}</a><br/>\n"));
  }
  html.push_str("</body></html>\n");
  Ok(Html(html).into_response())
}

#[utoipa::path(
    get,path="/projects/{id}/packages/pypi/simple/{name}",tag="package protocols",
    params(api::PackageNamePath),
    responses((status=200,description="Python Simple file index",body=String),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn simple_package(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path((project_id, name)): Path<(i64, String)>,
) -> Result<Response, AppError> {
  authorize(&state, &headers, project_id, PackageAction::Read).await?;
  let detail = named(&state, project_id, "pypi", &name).await?;
  let mut files = Vec::new();
  for version in detail["versions"].as_array().into_iter().flatten() {
    for file in version["files"].as_array().into_iter().flatten() {
      let mut value = json!({
          "filename":text(file,"file_name"),
          "url":format!("/api/v1/projects/{project_id}/packages/files/{}/download/{}#sha256={}",num(file,"id"),text(file,"file_name"),text(file,"sha256")),
          "hashes":{"sha256":text(file,"sha256")},"yanked":false
      });
      if let Ok(metadata) = serde_json::from_str::<Value>(text(file, "registry_metadata"))
        && !metadata["requires_python"].is_null()
      {
        value["requires-python"] = metadata["requires_python"].clone();
      }
      files.push((file, value));
    }
  }
  if accepts_json(&headers) {
    return Ok(
      (
        [("content-type", "application/vnd.pypi.simple.v1+json")],
        axum::Json(json!({
            "meta":{"api-version":"1.0"},"name":name,
            "files":files.iter().map(|(_, value)| value.clone()).collect::<Vec<_>>()
        })),
      )
        .into_response(),
    );
  }
  let mut html = String::from("<!doctype html><html><body>\n");
  for (file, _) in files {
    html.push_str(&format!(
            "<a href=\"/api/v1/projects/{project_id}/packages/files/{}/download/{}#sha256={}\">{}</a><br/>\n",
            num(file, "id"), escape(text(file, "file_name")), escape(text(file, "sha256")), escape(text(file, "file_name"))
        ));
  }
  html.push_str("</body></html>\n");
  Ok(Html(html).into_response())
}

fn accepts_json(headers: &HeaderMap) -> bool {
  headers
    .get(axum::http::header::ACCEPT)
    .and_then(|value| value.to_str().ok())
    .is_some_and(|value| value.contains("application/vnd.pypi.simple.v1+json"))
}

fn escape(value: impl AsRef<str>) -> String {
  value
    .as_ref()
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}
