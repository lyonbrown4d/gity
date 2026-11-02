//! Ingests the archive a successful index job uploaded into the shared store.
use crate::AppState;
use crate::ci;
use gity_core::AppError;
use gity_persistence::model_value;
use serde_json::Value;

pub(crate) async fn ingest(s: &AppState, project_id: i64, job: &Value) -> Result<(), AppError> {
  let job_id = job["id"]
    .as_i64()
    .filter(|id| *id > 0)
    .ok_or_else(|| AppError::internal("job row is missing its identifier"))?;
  let data = ci::payload(job)?;
  let name = data["artifact_name"]
    .as_str()
    .unwrap_or(super::ARTIFACT_NAME);
  let artifact = s
    .ci
    .artifacts(project_id, job_id)
    .await?
    .into_iter()
    .map(model_value)
    .find(|row| row["file_name"].as_str() == Some(name) || row["name"].as_str() == Some(name))
    .ok_or_else(|| AppError::not_found("code index artifact was not uploaded"))?;
  let key = artifact["storage_key"]
    .as_str()
    .unwrap_or_default()
    .to_owned();
  if !ci::config::safe_relative(&key) {
    return Err(AppError::internal("invalid artifact storage key"));
  }
  let bytes = s.storage.get(&key).await?;
  let archive = super::blocking({
    let bytes = bytes.clone();
    move || gity_code_intel::decode(&bytes)
  })
  .await?;
  let commit = archive.commit.clone();
  let files = archive.files.len();
  let definitions = archive.definition_count();
  let store = s.code_index.clone();
  super::blocking(move || store.refresh(project_id, &archive)).await?;
  // Mirror the exact bytes the runner uploaded: they are already the
  // msgpack+zstd archive `decode` accepts, so the shared copy needs no re-encode
  // and stays byte-identical to the CI artifact.
  super::publish(s, project_id, &commit, bytes).await;
  tracing::info!(
    project_id,
    job_id,
    commit = %commit,
    files,
    definitions,
    "code index ingested"
  );
  Ok(())
}
