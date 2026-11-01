use super::packages::named;
use super::*;
use crate::openapi::schemas as api;
use axum::extract::{Path, State};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(registration))
    .routes(utoipa_axum::routes!(registration_version))
}

#[utoipa::path(
  get,path="/projects/{id}/packages/nuget/registration/{name}/index.json",tag="package protocols",
  params(api::ProjectPath,api::PackageNamePath),
  responses((status=200,description="NuGet registration metadata"),api::Errors),
  security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn registration(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path((project_id, name)): Path<(i64, String)>,
) -> Result<Json<Value>, AppError> {
  authorize(&state, &headers, project_id, PackageAction::Read).await?;
  let normalized = name.to_ascii_lowercase();
  let detail = named(&state, project_id, "nuget", &normalized).await?;
  let base = format!(
    "{}/api/v1/projects/{project_id}/packages/nuget/registration/{normalized}",
    request_origin(&headers)?
  );
  let flat = format!(
    "{}/api/v1/projects/{project_id}/packages/nuget/flatcontainer/{normalized}",
    request_origin(&headers)?
  );
  let mut items = Vec::new();
  for version in detail["versions"].as_array().into_iter().flatten() {
    if text(version, "status") == "unlisted" {
      continue;
    }
    let version_name = text(&version["version"], "version");
    let Some(file) = version["files"]
      .as_array()
      .into_iter()
      .flatten()
      .find(|file| text(file, "file_name").ends_with(".nupkg"))
    else {
      continue;
    };
    let package_content = format!("{flat}/{version_name}/{normalized}.{version_name}.nupkg");
    let package_metadata = registry_metadata(&state, file).await?;
    let dependency_groups = package_metadata["dependency_groups"]
      .as_array()
      .into_iter()
      .flatten()
      .map(dependency_group)
      .collect::<Vec<_>>();
    items.push(json!({
      "@id":format!("{base}/{version_name}.json"),
      "catalogEntry":{
        "@id":format!("{base}/{version_name}.json"),"id":normalized,
        "version":version_name,"listed":true,"dependencyGroups":dependency_groups,
        "packageContent":package_content
      },
      "packageContent":package_content,"registration":format!("{base}/index.json")
    }));
  }
  Ok(Json(json!({
    "@id":format!("{base}/index.json"),"count":1,
    "items":[{"@id":format!("{base}/page.json"),"count":items.len(),"items":items}]
  })))
}

#[utoipa::path(
  get,path="/projects/{id}/packages/nuget/registration/{name}/{version}",tag="package protocols",
  params(api::ProjectPath,api::PackageNamePath,("version"=String,Path)),
  responses((status=200,description="NuGet package registration leaf"),api::Errors),
  security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn registration_version(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path((project_id, name, version)): Path<(i64, String, String)>,
) -> Result<Json<Value>, AppError> {
  let version = version.strip_suffix(".json").unwrap_or(&version);
  let index = registration(State(state), headers, Path((project_id, name)))
    .await?
    .0;
  index["items"][0]["items"]
    .as_array()
    .into_iter()
    .flatten()
    .find(|item| text(&item["catalogEntry"], "version") == version)
    .cloned()
    .map(Json)
    .ok_or_else(|| AppError::not_found("NuGet package version not found"))
}

async fn registry_metadata(state: &AppState, file: &Value) -> Result<Value, AppError> {
  if let Ok(metadata) = serde_json::from_str(text(file, "registry_metadata")) {
    return Ok(metadata);
  }
  let bytes = state
    .artifacts
    .storage
    .get(text(file, "storage_key"))
    .await?;
  let metadata = gity_artifacts::nuget_metadata(&bytes)?;
  Ok(json!({
    "dependency_groups":metadata.dependency_groups.iter().map(|group| json!({
      "target_framework":group.target_framework,
      "dependencies":group.dependencies.iter().map(|dependency| json!({
        "id":dependency.id,"range":dependency.range
      })).collect::<Vec<_>>()
    })).collect::<Vec<_>>()
  }))
}

fn dependency_group(group: &Value) -> Value {
  let mut value = json!({
    "dependencies":group["dependencies"].as_array().cloned().unwrap_or_default()
  });
  if !group["target_framework"].is_null() {
    value["targetFramework"] = group["target_framework"].clone();
  }
  value
}
