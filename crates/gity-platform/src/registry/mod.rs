//! Registry credentials, package distribution and Git LFS.
mod audit;
pub use audit::{AuditRecord, record_event};
mod credential_repository;
mod credentials;
mod lfs;
mod lfs_protocol;
pub use lfs_protocol::documented_routes as lfs_protocol_routes;
mod lfs_locks;
mod lfs_objects;
mod tokens;
pub use lfs::{handle_lfs, is_lfs_path};
mod variables;
use crate::{AppState, access::ProjectAction};
use axum::{Json, Router, http::HeaderMap};
use gity_core::{AppError, externalize, now};
use serde_json::{Map, Value, json};
pub use tokens::{ProjectToken, authenticate_token, token_from_headers};
type Reply = Result<Json<Value>, AppError>;
pub fn routes() -> Router<AppState> {
  documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  credentials::documented_routes()
    .merge(variables::documented_routes())
    .merge(audit::documented_routes())
    .merge(lfs::documented_routes())
}

fn reply(v: Value) -> Reply {
  Ok(gity_core::ok(externalize(v)))
}
fn num(v: &Value, k: &str) -> i64 {
  v[k]
    .as_i64()
    .or_else(|| v[k].as_str().and_then(|v| v.parse().ok()))
    .unwrap_or(0)
}
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
  v[k].as_str().unwrap_or("")
}
fn required(v: &Value, k: &str) -> Result<String, AppError> {
  let s = text(v, k).trim();
  if s.is_empty() {
    Err(AppError::bad_request(format!("{k} is required")))
  } else {
    Ok(s.to_owned())
  }
}
fn map(v: Value) -> Map<String, Value> {
  v.as_object().cloned().unwrap_or_default()
}

async fn scoped(s: &AppState, table: &str, p: i64, id: i64) -> Result<Value, AppError> {
  s.db
    .one(
      &format!("SELECT * FROM {table} WHERE project_id = ? AND id = ?"),
      vec![p.into(), id.into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("resource not found"))
}
async fn authorize(
  s: &AppState,
  h: &HeaderMap,
  p: i64,
  action: ProjectAction,
) -> Result<(), AppError> {
  if matches!(action, ProjectAction::PackageRead)
    && crate::ci::package_identity(s, h, p, false).await?.is_some()
  {
    s.identity
      .active_project(p)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))?;
    return Ok(());
  }
  crate::access::project_action(s, h, p, action).await?;
  Ok(())
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod credential_tests;
#[cfg(test)]
mod lfs_tests;
#[cfg(test)]
#[cfg(feature = "packages")]
mod package_tests;
fn timestamp(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
  chrono::DateTime::parse_from_rfc3339(value)
    .ok()
    .map(|v| v.to_utc())
    .or_else(|| {
      ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"]
        .iter()
        .find_map(|format| {
          chrono::NaiveDateTime::parse_from_str(value, format)
            .ok()
            .map(|v| v.and_utc())
        })
    })
}
