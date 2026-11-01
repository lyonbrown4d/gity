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
use sea_orm::{ConnectionTrait, DatabaseTransaction, FromQueryResult, Value as SqlValue};
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
pub(crate) async fn one(
  s: &AppState,
  table: &str,
  project: i64,
  item: i64,
) -> Result<Value, AppError> {
  s.db
    .one(
      &format!("SELECT * FROM {table} WHERE project_id=? AND id=?"),
      vec![project.into(), item.into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("CI resource not found"))
}
pub(crate) async fn insert_tx(
  s: &AppState,
  tx: &DatabaseTransaction,
  table: &str,
  mut row: Value,
) -> Result<Value, AppError> {
  let row = row
    .as_object_mut()
    .ok_or_else(|| AppError::internal("invalid CI row"))?;
  row.entry("id").or_insert(json!(new_id()));
  row.entry("created_at").or_insert(json!(now()));
  row.entry("updated_at").or_insert(json!(now()));
  let columns = row.keys().cloned().collect::<Vec<_>>();
  let values = row
    .values()
    .map(|v| match v {
      Value::String(v) => Ok(SqlValue::from(v.clone())),
      Value::Number(v) => v
        .as_i64()
        .map(SqlValue::from)
        .ok_or_else(|| AppError::internal("invalid CI integer")),
      _ => Err(AppError::internal("invalid CI field")),
    })
    .collect::<Result<Vec<_>, _>>()?;
  tx.execute_raw(s.db.statement(
    &format!(
      "INSERT INTO {table} ({}) VALUES ({})",
      columns.join(","),
      vec!["?"; columns.len()].join(",")
    ),
    values,
  ))
  .await?;
  Ok(Value::Object(row.clone()))
}
pub(crate) async fn tx_rows(
  s: &AppState,
  tx: &DatabaseTransaction,
  sql: &str,
  args: Vec<SqlValue>,
) -> Result<Vec<Value>, AppError> {
  Ok(
    Value::find_by_statement(s.db.statement(sql, args))
      .all(tx)
      .await?,
  )
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
