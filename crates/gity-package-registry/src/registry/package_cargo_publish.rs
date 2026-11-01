use super::package_storage::{Upload, upload};
use super::*;
use crate::openapi::schemas as api;
use axum::{
  body::Body,
  extract::{Path, State},
  http::HeaderMap,
};
use bytes::{Bytes, BytesMut};
use futures_util::{
  StreamExt,
  stream::{self, BoxStream},
};
use std::collections::VecDeque;

const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_CRATE_BYTES: u64 = 1024 * 1024 * 1024;

#[utoipa::path(
    put,path="/projects/{id}/packages/cargo/api/v1/crates/new",tag="package protocols",
    params(api::ProjectPath),
    request_body(content_type="application/octet-stream"),
    responses((status=200,description="Cargo crate published"),api::Errors),
    security(("private_token"=[]),("basic"=[]))
)]
pub(super) async fn publish(
  State(s): State<AppState>,
  mut h: HeaderMap,
  Path(project_id): Path<i64>,
  body: Body,
) -> Result<axum::Json<Value>, AppError> {
  super::package_cargo::normalize_cargo_token(&mut h)?;
  let actor = authorize_package_write(&s, &h, project_id).await?;
  let (metadata, crate_body) = CargoPublishBody::parse(body).await?;
  let name = gity_artifacts::PackageFormat::Cargo.normalize_name(text(&metadata, "name"))?;
  let version = text(&metadata, "vers");
  if version.is_empty() || semver::Version::parse(version).is_err() {
    return Err(AppError::bad_request("Cargo crate version is invalid"));
  }
  let path = format!("{name}-{version}.crate");
  let registry_metadata = json!({
    "name":name,
    "version":version,
    "dependencies":index_dependencies(&metadata),
    "features":metadata["features"].as_object().cloned().unwrap_or_default(),
    "links":metadata["links"],
    "rust_version":metadata["rust_version"]
  })
  .to_string();
  let result = upload(
    &s,
    project_id,
    Upload {
      kind: "cargo",
      name: &name,
      version,
      path: &path,
      content_type: "application/gzip",
      created_by_user_id: actor.user_id,
      source: &actor.source,
      pipeline_id: actor.pipeline_id,
      job_id: actor.job_id,
      registry_metadata: Some(registry_metadata),
    },
    Body::from_stream(crate_body),
  )
  .await?;
  Ok(axum::Json(json!({
    "ok":true,
    "warnings":{"invalid_categories":[],"invalid_badges":[],"other":[]},
    "crate_id":result["id"]
  })))
}

fn index_dependencies(metadata: &Value) -> Vec<Value> {
  metadata["deps"]
    .as_array()
    .into_iter()
    .flatten()
    .map(|dependency| {
      let mut index_entry = dependency.clone();
      if let Some(requirement) = dependency.get("version_req") {
        index_entry["req"] = requirement.clone();
      }
      if let Some(entry) = index_entry.as_object_mut() {
        entry.remove("version_req");
        if let Some(alias) = entry.remove("explicit_name_in_toml") {
          entry.insert("package".into(), alias);
        }
      }
      index_entry
    })
    .collect()
}

struct CargoPublishBody {
  stream: BoxStream<'static, Result<Bytes, AppError>>,
  pending: VecDeque<Bytes>,
}

impl CargoPublishBody {
  async fn parse(
    body: Body,
  ) -> Result<
    (
      Value,
      impl futures_util::Stream<Item = Result<Bytes, AppError>> + Send + 'static,
    ),
    AppError,
  > {
    let stream = body
      .into_data_stream()
      .map(|chunk| chunk.map_err(|_| AppError::bad_request("invalid Cargo publish body")))
      .boxed();
    let mut input = Self {
      stream,
      pending: VecDeque::new(),
    };
    let metadata_length =
      u32::from_le_bytes(input.read_exact(4).await?.as_ref().try_into().unwrap()) as usize;
    if metadata_length == 0 || metadata_length > MAX_METADATA_BYTES {
      return Err(AppError::bad_request("Cargo publish metadata is too large"));
    }
    let metadata: Value = serde_json::from_slice(&input.read_exact(metadata_length).await?)
      .map_err(|_| AppError::bad_request("invalid Cargo publish metadata"))?;
    let crate_length =
      u32::from_le_bytes(input.read_exact(4).await?.as_ref().try_into().unwrap()) as u64;
    if crate_length == 0 || crate_length > MAX_CRATE_BYTES {
      return Err(AppError::bad_request("Cargo crate archive size is invalid"));
    }
    let payload = input.into_payload(crate_length);
    Ok((metadata, payload))
  }

  async fn read_exact(&mut self, length: usize) -> Result<Bytes, AppError> {
    let mut output = BytesMut::with_capacity(length);
    while output.len() < length {
      while let Some(chunk) = self.pending.pop_front() {
        let needed = length - output.len();
        let take = needed.min(chunk.len());
        output.extend_from_slice(&chunk[..take]);
        if take < chunk.len() {
          self.pending.push_front(chunk.slice(take..));
          return Ok(output.freeze());
        }
        if output.len() == length {
          return Ok(output.freeze());
        }
      }
      let chunk = self
        .stream
        .next()
        .await
        .ok_or_else(|| AppError::bad_request("truncated Cargo publish body"))??;
      self.pending.push_back(chunk);
    }
    Ok(output.freeze())
  }

  fn into_payload(
    self,
    length: u64,
  ) -> impl futures_util::Stream<Item = Result<Bytes, AppError>> + Send + 'static {
    let input = stream::iter(self.pending.into_iter().map(Ok))
      .chain(self.stream)
      .boxed();
    stream::try_unfold(
      (input, VecDeque::new(), length),
      |(mut input, mut pending, remaining)| async move {
        if remaining == 0 {
          if !pending.is_empty() || input.next().await.is_some() {
            return Err(AppError::bad_request(
              "trailing bytes in Cargo publish body",
            ));
          }
          return Ok(None);
        }
        while pending.is_empty() {
          pending.push_back(
            input
              .next()
              .await
              .ok_or_else(|| AppError::bad_request("truncated Cargo crate archive"))??,
          );
        }
        let chunk = pending.pop_front().unwrap();
        let take = remaining.min(chunk.len() as u64) as usize;
        let output = chunk.slice(..take);
        if take < chunk.len() {
          pending.push_front(chunk.slice(take..));
        }
        Ok(Some((output, (input, pending, remaining - take as u64))))
      },
    )
  }
}

#[cfg(test)]
#[path = "package_cargo_publish_tests.rs"]
mod tests;
