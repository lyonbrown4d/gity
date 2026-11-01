use super::*;
use axum::body::Body;
use futures_util::StreamExt;
use gity_artifacts::{ArtifactCoordinate, PackageFormat};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
pub(super) struct Upload<'a> {
  pub kind: &'a str,
  pub name: &'a str,
  pub version: &'a str,
  pub path: &'a str,
  pub content_type: &'a str,
  pub created_by_user_id: Option<i64>,
  pub source: &'a str,
  pub pipeline_id: Option<i64>,
  pub job_id: Option<i64>,
  pub registry_metadata: Option<String>,
}
pub(super) async fn upload(
  s: &AppState,
  p: i64,
  input: Upload<'_>,
  body: Body,
) -> Result<Value, AppError> {
  if [input.kind, input.name, input.version, input.path]
    .iter()
    .any(|v| v.trim().is_empty())
  {
    return Err(AppError::bad_request(
      "type, name, version, and file_name are required",
    ));
  }
  let format = PackageFormat::parse(input.kind)
    .ok_or_else(|| AppError::bad_request("unsupported package format"))?;
  let coordinate = ArtifactCoordinate::new(p, format, input.name, input.version, input.path)?;
  let path = coordinate.path.clone();
  let name = path.rsplit('/').next().unwrap_or(&path);
  if name.is_empty() {
    return Err(AppError::bad_request("file_name is required"));
  }
  let id = gity_core::new_id();
  let key = format!("packages/{p}/{id}");
  let digest = Arc::new(Mutex::new(Sha256::new()));
  let digest_stream = Arc::clone(&digest);
  let size = s
    .storage
    .put_stream(
      &key,
      body.into_data_stream().map(move |v| {
        v.inspect(|chunk| {
          digest_stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .update(chunk);
        })
        .map_err(|_| AppError::bad_request("read package request body failed"))
      }),
      1024 * 1024 * 1024,
    )
    .await?;
  let sha256 = hex::encode(
    digest
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .clone()
      .finalize(),
  );
  let result = async {
    let (row, _) = s
      .artifacts
      .repository
      .record_upload(gity_artifacts::RecordArtifactUpload {
        coordinate: &coordinate,
        file_name: name,
        content_type: if input.content_type.is_empty() {
          "application/octet-stream"
        } else {
          input.content_type
        },
        byte_size: size,
        sha256: &sha256,
        storage_key: &key,
        created_by_user_id: input.created_by_user_id,
        source: input.source,
        pipeline_id: input.pipeline_id,
        job_id: input.job_id,
        registry_metadata: input.registry_metadata.as_deref(),
      })
      .await?;
    Ok(file_value(&row))
  }
  .await;
  if !matches!(&result, Ok(row) if text(row, "storage_key") == key) {
    let _ = s.storage.delete(&key).await;
  }
  result
}
pub(super) fn file_value(row: &gity_entities::project_package_files::Model) -> Value {
  json!({
      "id": row.id,
      "project_package_version_id": row.project_package_version_id,
      "file_name": row.file_name,
      "file_path": row.file_path,
      "content_type": row.content_type,
      "byte_size": row.byte_size,
      "sha256": row.sha256,
      "created_by_user_id": row.created_by_user_id,
      "pipeline_id": row.pipeline_id,
      "job_id": row.job_id,
      "source": row.source,
      "storage_key": row.storage_key,
      "created_at": row.created_at,
      "updated_at": row.updated_at,
  })
}
pub(super) fn protocol_file_value(row: &gity_entities::project_package_files::Model) -> Value {
  let mut value = file_value(row);
  value["registry_metadata"] = row
    .registry_metadata
    .clone()
    .map_or(Value::Null, Value::String);
  value
}
