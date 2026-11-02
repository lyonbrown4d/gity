mod artifacts;
#[cfg(test)]
mod capacity_tests;
mod claims;
pub mod config;
pub mod job_schemas;
mod jobs;
mod package_identity;
#[cfg(feature = "oci")]
pub(crate) use package_identity::authenticate as authenticate_job_credential;
#[cfg(test)]
mod package_identity_tests;
pub use package_identity::{JobPackageIdentity, package_identity};
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod openapi_tests;
pub mod params;
pub mod pipeline_schemas;
mod pipelines;
pub mod runner_schemas;
mod runners;
#[cfg(test)]
mod security_tests;
mod service;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod trace;
mod variables;
mod views;
mod worker;
use crate::AppState;
use crate::access::ProjectAction;
use axum::{Json, Router};
use gity_core::{AppError, new_id, now};
use gity_persistence::model_value;
use serde_json::{Value, json};
pub use service::{CreatePipelineInput, create_pipeline, create_push_pipeline};
pub use worker::{refresh_pipeline, worker_tick};
pub type Reply = Result<Json<Value>, AppError>;
pub fn routes() -> Router<AppState> {
  documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let project = utoipa_axum::router::OpenApiRouter::new()
    .merge(pipelines::documented_routes())
    .merge(jobs::documented_routes())
    .merge(runners::project_routes());
  utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", project.clone())
    .nest("/repos", project)
    .merge(runners::agent_routes())
}
pub(crate) fn flag(row: &Value, key: &str) -> bool {
  row[key]
    .as_bool()
    .unwrap_or_else(|| row[key].as_i64().is_some_and(|value| value != 0))
}
pub(crate) fn id(row: &Value) -> i64 {
  row["id"].as_i64().unwrap_or_default()
}
pub(crate) fn text<'a>(row: &'a Value, key: &str) -> &'a str {
  row[key].as_str().unwrap_or_default()
}
pub(crate) fn payload(row: &Value) -> Result<Value, AppError> {
  serde_json::from_str(text(row, "payload"))
    .map_err(|_| AppError::internal("invalid persisted CI job payload"))
}
pub(crate) fn git_error(e: anyhow::Error) -> AppError {
  AppError::bad_request(e.to_string())
}
pub(crate) async fn job_row(s: &AppState, project: i64, item: i64) -> Result<Value, AppError> {
  Ok(model_value(s.ci.job(project, item).await?))
}
pub(crate) async fn pipeline_row(s: &AppState, project: i64, item: i64) -> Result<Value, AppError> {
  Ok(model_value(s.ci.pipeline(project, item).await?))
}
pub(crate) fn lease(job: &Value) -> gity_ci::Lease {
  gity_ci::Lease {
    job_id: id(job),
    attempt: job["attempts"].as_i64().unwrap_or(0) as i32,
    owner: text(job, "locked_by").into(),
  }
}
pub(crate) fn public_job(mut row: Value) -> Value {
  if let Ok(mut body) = payload(&row) {
    if let Some(map) = body.as_object_mut() {
      map.remove("env");
      map.remove("masked_values");
      map.remove("trace_pending");
      map.remove(package_identity::HASH_FIELD);
    }
    row["payload"] = json!(body.to_string());
  }
  row
}
