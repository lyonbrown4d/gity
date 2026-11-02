use crate::AppState;
use axum::http::HeaderMap;
use gity_core::AppError;
use serde_json::{Value, json};
use subtle::ConstantTimeEq;

const PREFIX: &str = "gity_job_v1.";
pub(crate) const HASH_FIELD: &str = "package_credential_hash";

#[derive(Clone, Debug)]
pub struct JobPackageIdentity {
  pub project_id: i64,
  pub job_id: i64,
  pub pipeline_id: Option<i64>,
  pub actor_id: Option<i64>,
}

pub(crate) fn issue(job: i64, attempt: i64) -> (String, String) {
  let secret = format!(
    "{}{}",
    uuid::Uuid::new_v4().simple(),
    uuid::Uuid::new_v4().simple()
  );
  let token = format!("{PREFIX}{job}.{attempt}.{secret}");
  let digest = crate::auth::token_hash(&token);
  (token, digest)
}

pub async fn package_identity(
  state: &AppState,
  headers: &HeaderMap,
  project: i64,
  _write: bool,
) -> Result<Option<JobPackageIdentity>, AppError> {
  let explicit = headers.get("job-token");
  let raw = if let Some(value) = explicit {
    Some(value.to_str().map_err(|_| denied())?.to_owned())
  } else {
    crate::registry::token_from_headers(headers).filter(|token| token.starts_with(PREFIX))
  };
  let Some(raw) = raw else {
    return Ok(None);
  };
  let identity = authenticate(state, &raw).await?;
  if identity.project_id != project {
    return Err(AppError::forbidden("job token belongs to another project"));
  }
  Ok(Some(identity))
}

pub(crate) async fn authenticate(
  state: &AppState,
  raw: &str,
) -> Result<JobPackageIdentity, AppError> {
  let pieces = raw.split('.').collect::<Vec<_>>();
  if pieces.len() != 4
    || pieces[0] != "gity_job_v1"
    || pieces[3].len() != 64
    || !pieces[3].bytes().all(|byte| byte.is_ascii_hexdigit())
  {
    return Err(denied());
  }
  let job_id: i64 = pieces[1].parse().map_err(|_| denied())?;
  let attempt: i64 = pieces[2].parse().map_err(|_| denied())?;
  if job_id <= 0 || attempt <= 0 {
    return Err(denied());
  }
  let job = state
    .ci
    .live_job(job_id, attempt as i32)
    .await?
    .ok_or_else(denied)?;
  if job.kind != "script"
    || !job
      .locked_by
      .as_deref()
      .unwrap_or_default()
      .starts_with("runner:")
  {
    return Err(denied());
  }
  let data: Value =
    serde_json::from_str(job.payload.as_deref().unwrap_or("{}")).map_err(|_| denied())?;
  let actual = crate::auth::token_hash(raw);
  let expected = data[HASH_FIELD].as_str().unwrap_or_default();
  if !bool::from(expected.as_bytes().ct_eq(actual.as_bytes())) {
    return Err(denied());
  }
  state
    .identity
    .active_project(job.project_id)
    .await?
    .ok_or_else(denied)?;
  let link = state.ci.job_link(job.id).await?;
  Ok(JobPackageIdentity {
    project_id: job.project_id,
    job_id: job.id,
    pipeline_id: link.map(|link| link.pipeline_id),
    actor_id: None,
  })
}

/// Add the transient credential only to the redaction copy, never persisted payload.
pub(crate) async fn redaction_job(
  state: &AppState,
  headers: &HeaderMap,
  mut job: Value,
) -> Result<Value, AppError> {
  let mut data = super::payload(&job)?;
  if data[HASH_FIELD].is_string() {
    let raw = headers
      .get("job-token")
      .and_then(|value| value.to_str().ok())
      .ok_or_else(denied)?;
    let identity = authenticate(state, raw).await?;
    if identity.job_id != super::id(&job) {
      return Err(denied());
    }
    let mut masked = data["masked_values"]
      .as_array()
      .cloned()
      .unwrap_or_default();
    masked.push(json!(raw));
    data["masked_values"] = json!(masked);
    job["payload"] = json!(data.to_string());
  }
  Ok(job)
}
fn denied() -> AppError {
  AppError::unauthorized("invalid or expired CI job token")
}
