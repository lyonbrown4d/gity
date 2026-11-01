use super::package_storage::{Upload, upload};
use super::packages::named;
use super::*;
use crate::openapi::{package_schemas as spec, schemas as api};
use axum::{
  body::Body,
  extract::{Path, State},
  http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Map;
async fn npm_metadata(s: &AppState, p: i64, name: &str, detail: &Value, origin: &str) -> Value {
  let mut versions = Map::new();
  let mut latest: Option<semver::Version> = None;
  let mut dist_tags = Map::new();
  if let Ok(tags) = s.artifacts.repository.npm_dist_tags(p, name).await {
    for (tag, version) in tags {
      dist_tags.insert(tag, json!(version));
    }
  }
  for version in detail["versions"].as_array().into_iter().flatten() {
    let key = text(&version["version"], "version");
    if let Ok(parsed) = semver::Version::parse(key)
      && latest.as_ref().is_none_or(|current| parsed > *current)
    {
      latest = Some(parsed);
    }
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let file = files
      .iter()
      .find(|v| text(v, "file_name").ends_with(".tgz"))
      .or_else(|| files.first());
    let mut dist = file
      .map(|file| {
        json!({
                "tarball": format!("{origin}/api/v1/projects/{p}/packages/files/{}/download", num(file, "id"))
        })
      })
      .unwrap_or(json!({}));
    if let Some(file) = file
      && let Ok(integrity) = gity_artifacts::sha256_integrity(text(file, "sha256"))
    {
      dist["integrity"] = json!(integrity);
    }
    let manifest = if let Some(file) = file {
      serde_json::from_str(text(file, "registry_metadata")).unwrap_or(Value::Null)
    } else {
      Value::Null
    };
    let mut metadata = json!({"name":name,"version":key,"dist":dist});
    for field in [
      "dependencies",
      "optionalDependencies",
      "peerDependencies",
      "engines",
    ] {
      if let Some(value) = manifest.get(field) {
        metadata[field] = value.clone();
      }
    }
    versions.insert(key.into(), metadata);
  }
  if !dist_tags.contains_key("latest")
    && let Some(latest) = latest
  {
    dist_tags.insert("latest".into(), json!(latest.to_string()));
  }
  json!({"name":name,"dist-tags":dist_tags,"versions":versions})
}
#[utoipa::path(
    get,path="/projects/{id}/packages/npm/{*name}",tag="package protocols",
    params(api::PackageNamePath),
    responses((status=200,description="Registry protocol response",body=spec::NpmMetadata),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn npm(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, name)): Path<(i64, String)>,
) -> Result<Json<Value>, AppError> {
  authorize(&s, &h, p, PackageAction::Read).await?;
  Ok(Json(
    npm_metadata(
      &s,
      p,
      &name,
      &named(&s, p, "npm", &name).await?,
      &request_origin(&h)?,
    )
    .await,
  ))
}
#[utoipa::path(
    put,path="/projects/{id}/packages/npm/{*name}",tag="package protocols",
    params(api::PackageNamePath),
    request_body(content=spec::NpmPublish),
    responses((status=200,description="Registry protocol response",body=spec::NpmMetadata),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn npm_publish(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, path)): Path<(i64, String)>,
  Json(b): Json<Value>,
) -> Result<Json<Value>, AppError> {
  let actor = authorize_package_write(&s, &h, p).await?;
  let name = if text(&b, "name").is_empty() {
    path
  } else {
    text(&b, "name").to_owned()
  };
  let version = if text(&b["dist-tags"], "latest").is_empty() {
    b["versions"]
      .as_object()
      .and_then(|v| v.keys().max())
      .cloned()
      .unwrap_or_default()
  } else {
    text(&b["dist-tags"], "latest").into()
  };
  if name.is_empty() || version.is_empty() {
    return Err(AppError::bad_request(
      "npm package name and version are required",
    ));
  }
  let attachments = b["_attachments"]
    .as_object()
    .filter(|v| !v.is_empty())
    .ok_or_else(|| AppError::bad_request("npm package attachment is required"))?;
  let mut decoded = Vec::new();
  for (file, attachment) in attachments {
    decoded.push((
      file,
      attachment,
      STANDARD
        .decode(text(attachment, "data"))
        .map_err(|_| AppError::bad_request("npm package attachment is invalid"))?,
    ));
  }
  let mut uploaded = Value::Null;
  let registry_metadata = decoded
    .iter()
    .find(|(file, _, _)| file.ends_with(".tgz"))
    .and_then(|(_, _, bytes)| gity_artifacts::npm_manifest(bytes).ok())
    .map(|manifest| manifest.to_string());
  if let Some(serialized) = &registry_metadata
    && let Ok(manifest) = serde_json::from_str::<Value>(serialized)
    && (!text(&manifest, "name").is_empty() && text(&manifest, "name") != name
      || !text(&manifest, "version").is_empty() && text(&manifest, "version") != version)
  {
    return Err(AppError::bad_request(
      "npm tarball metadata does not match the publish request",
    ));
  }
  for (file, attachment, data) in decoded {
    uploaded = upload(
      &s,
      p,
      Upload {
        kind: "npm",
        name: &name,
        version: &version,
        path: file,
        content_type: text(attachment, "content_type"),
        created_by_user_id: actor.user_id,
        source: &actor.source,
        pipeline_id: actor.pipeline_id,
        job_id: actor.job_id,
        registry_metadata: registry_metadata.clone(),
      },
      Body::from(data),
    )
    .await?;
  }
  let mut tags = b["dist-tags"].as_object().cloned().unwrap_or_default();
  if !tags.contains_key("latest") {
    tags.insert("latest".into(), json!(version));
  }
  for (tag, tagged_version) in tags {
    let version = tagged_version
      .as_str()
      .ok_or_else(|| AppError::bad_request("npm dist-tag version must be a string"))?;
    s.artifacts
      .repository
      .set_npm_dist_tag(p, &name, &tag, Some(version))
      .await?;
  }
  let mut metadata = npm_metadata(
    &s,
    p,
    &name,
    &named(&s, p, "npm", &name).await?,
    &request_origin(&h)?,
  )
  .await;
  metadata["uploaded_file_id"] = json!(num(&uploaded, "id").to_string());
  Ok(Json(metadata))
}
#[utoipa::path(
    get,path="/projects/{id}/packages/nuget/index.json",tag="package protocols",
    params(api::ProjectPath),
    responses((status=200,description="Registry protocol response",body=spec::NugetIndex),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn nuget(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
) -> Result<Json<Value>, AppError> {
  authorize(&s, &h, p, PackageAction::Read).await?;
  let base = format!("{}/api/v1/projects/{p}/packages/nuget", request_origin(&h)?);
  Ok(Json(json!({"version":"3.0.0","resources":[
      {"@id":format!("{base}/push"),"@type":"PackagePublish/2.0.0"},
      {"@id":format!("{base}/flatcontainer/"),"@type":"PackageBaseAddress/3.0.0"},
      {"@id":format!("{base}/query"),"@type":"SearchQueryService/3.5.0"},
      {"@id":format!("{base}/registration/"),"@type":"RegistrationsBaseUrl/3.6.0"}
  ]})))
}
