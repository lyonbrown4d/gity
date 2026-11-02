mod protection;
mod lookup;
mod read;
mod read_routes;
mod search;
mod views;
mod write;
mod write_routes;
use crate::{AppState, access, auth, projects};
use axum::{
  Json, Router,
  extract::{Path, Query, State},
  http::HeaderMap,
};
use gity_core::{AppError, ok};
pub use protection::{check_branch, effective_rule, matches_rule};
use serde_json::{Value, json};
use std::collections::HashMap;
type Reply = Result<Json<Value>, AppError>;
pub fn routes() -> Router<AppState> {
  documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  use utoipa_axum::{router::OpenApiRouter, routes};
  let shared = lookup::routes()
    .merge(read_routes::routes())
    .merge(write_routes::routes())
    .routes(routes!(protection::protect))
    .routes(routes!(protection::update))
    .routes(routes!(protection::unprotect));
  let refresh = OpenApiRouter::new()
    .routes(routes!(search::refresh))
    .merge(crate::code_index::routes());
  let mut router = OpenApiRouter::new()
    .routes(routes!(read_routes::tags))
    .nest("/projects/{id}/repository", shared.clone())
    .nest("/repos/{id}", shared)
    .nest("/projects", refresh.clone())
    .nest("/repos", refresh);
  for (source, target) in [
    (
      "/projects/{id}/repository/languages",
      "/projects/{id}/languages",
    ),
    (
      "/projects/{id}/repository/file-commits",
      "/projects/{id}/file-commits",
    ),
  ] {
    crate::openapi::alias(&mut router, source, target);
  }
  router
    .route(
      "/projects/{id}/languages",
      axum::routing::get(read_routes::languages),
    )
    .route(
      "/projects/{id}/file-commits",
      axum::routing::post(write_routes::file_commits),
    )
}
pub fn repo(project: &Value) -> String {
  // Persisted project IDs are immutable; namespace changes only affect URLs.
  let id =
    projects::id(&project["id"]).expect("persisted project row must contain a positive identifier");
  format!("{id}.git")
}
pub fn default_branch(project: &Value) -> &str {
  project["default_branch"].as_str().unwrap_or("main")
}
pub fn git_error(e: anyhow::Error) -> AppError {
  let text = e.to_string();
  if text.contains("not found") || text.contains("empty") {
    AppError::not_found(text)
  } else if text.contains("already exists") || text.contains("conflict") || text.contains("changed")
  {
    AppError::conflict(text)
  } else {
    tracing::warn!(error=%e,"repository operation failed");
    AppError::bad_request("repository operation failed")
  }
}
