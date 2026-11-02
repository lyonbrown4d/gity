use super::packages::{coordinate, named};
use super::*;
use crate::openapi::schemas as api;
use axum::{
  extract::{Path, State},
  http::{HeaderMap, HeaderValue, header},
  response::{IntoResponse, Response},
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(config))
    .routes(utoipa_axum::routes!(index))
    .routes(utoipa_axum::routes!(super::package_cargo_publish::publish))
    .routes(utoipa_axum::routes!(download))
}

#[utoipa::path(
    get,path="/projects/{id}/packages/cargo/index/config.json",tag="package protocols",
    params(api::ProjectPath),
    responses((status=200,description="Cargo sparse registry configuration"),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn config(
  State(s): State<AppState>,
  mut h: HeaderMap,
  Path(project_id): Path<i64>,
) -> Result<axum::Json<Value>, AppError> {
  normalize_cargo_token(&mut h)?;
  let actor = authorize(&s, &h, project_id, PackageAction::Read).await?;
  let origin = request_origin(&h)?;
  let base = format!("{origin}/api/v1/projects/{project_id}/packages/cargo");
  Ok(axum::Json(json!({
      "dl":format!("{base}/crates/{{crate}}/{{version}}/download"),
      "api":base,
      "auth-required":!actor.project_public
  })))
}

#[utoipa::path(
    get,path="/projects/{id}/packages/cargo/index/{*index_path}",tag="package protocols",
    params(("id"=i64,Path),("index_path"=String,Path)),
    responses((status=200,description="Cargo sparse index JSON lines",content_type="text/plain"),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn index(
  State(s): State<AppState>,
  mut h: HeaderMap,
  Path((project_id, index_path)): Path<(i64, String)>,
) -> Result<Response, AppError> {
  normalize_cargo_token(&mut h)?;
  authorize(&s, &h, project_id, PackageAction::Read).await?;
  let name = index_path.rsplit('/').next().unwrap_or(&index_path);
  let normalized = gity_artifacts::PackageFormat::Cargo.normalize_name(name)?;
  if gity_artifacts::cargo_index_path(&normalized)? != index_path {
    return Err(AppError::not_found(
      "Cargo crate index path does not match crate name",
    ));
  }
  let detail = named(&s, project_id, "cargo", &normalized).await?;
  let mut lines = String::new();
  for version in detail["versions"].as_array().into_iter().flatten() {
    let version_name = text(&version["version"], "version");
    let Some(file) = version["files"]
      .as_array()
      .into_iter()
      .flatten()
      .find(|file| text(file, "file_name").ends_with(".crate"))
    else {
      continue;
    };
    let metadata = if let Ok(value) = serde_json::from_str::<Value>(text(file, "registry_metadata"))
    {
      value
    } else {
      let bytes = s.storage.get(text(file, "storage_key")).await?;
      let parsed = gity_artifacts::cargo_metadata(&bytes)?;
      json!({
          "name":parsed.name,
          "version":parsed.version,
          "dependencies":parsed.dependencies,
          "features":parsed.features,
          "links":parsed.links,
          "rust_version":parsed.rust_version
      })
    };
    let all_features = metadata["features"]
      .as_object()
      .cloned()
      .unwrap_or_default();
    let mut legacy_features = serde_json::Map::new();
    let mut namespaced_features = serde_json::Map::new();
    for (feature, values) in all_features {
      let namespaced = values.as_array().into_iter().flatten().any(|value| {
        value
          .as_str()
          .is_some_and(|entry| entry.contains("dep:") || entry.contains('/'))
      });
      if namespaced {
        namespaced_features.insert(feature, values);
      } else {
        legacy_features.insert(feature, values);
      }
    }
    let mut row = json!({
        "name":normalized,
        "vers":version_name,
        "deps":metadata["dependencies"],
        "cksum":text(file, "sha256"),
        "features":legacy_features,
        "yanked":text(version, "status") == "yanked",
        "v":2
    });
    if !namespaced_features.is_empty() {
      row["features2"] = json!(namespaced_features);
    }
    if !metadata["links"].is_null() {
      row["links"] = metadata["links"].clone();
    }
    if !metadata["rust_version"].is_null() {
      row["rust_version"] = metadata["rust_version"].clone();
    }
    lines.push_str(
      &serde_json::to_string(&row)
        .map_err(|_| AppError::internal("failed to serialize Cargo index entry"))?,
    );
    lines.push('\n');
  }
  Ok(
    (
      [(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
      )],
      lines,
    )
      .into_response(),
  )
}

#[utoipa::path(
    get,path="/projects/{id}/packages/cargo/crates/{name}/{version}/download",tag="package protocols",
    params(api::PackageNamePath,("version"=String, Path, description="Crate version")),
    responses((status=200,description="Cargo crate archive",body=api::Binary,content_type="application/gzip"),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download(
  State(s): State<AppState>,
  mut h: HeaderMap,
  Path((project_id, name, version)): Path<(i64, String, String)>,
) -> Result<Response, AppError> {
  normalize_cargo_token(&mut h)?;
  authorize(&s, &h, project_id, PackageAction::Read).await?;
  let name = gity_artifacts::PackageFormat::Cargo.normalize_name(&name)?;
  let path = format!("{name}-{version}.crate");
  super::package_binary::response(
    &s,
    &h,
    coordinate(&s, project_id, "cargo", &name, &version, &path).await?,
  )
  .await
}

pub(super) fn normalize_cargo_token(headers: &mut HeaderMap) -> Result<(), AppError> {
  if headers.contains_key("private-token") {
    return Ok(());
  }
  let Some(value) = headers
    .get(header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
  else {
    return Ok(());
  };
  let token = match value.split_once(' ') {
    Some((scheme, token)) if scheme.eq_ignore_ascii_case("token") => token.trim(),
    Some((scheme, _))
      if scheme.eq_ignore_ascii_case("bearer") || scheme.eq_ignore_ascii_case("basic") =>
    {
      return Ok(());
    }
    None => value.trim(),
    _ => return Ok(()),
  };
  if !token.is_empty() {
    headers.insert(
      "private-token",
      HeaderValue::from_str(token).map_err(|_| AppError::unauthorized("invalid Cargo token"))?,
    );
  }
  Ok(())
}
