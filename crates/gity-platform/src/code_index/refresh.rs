//! Synchronous rebuild of a project's code index from its default-branch HEAD.
use super::snapshot;
use crate::AppState;
use crate::access::{self, ProjectAction};
use crate::openapi::schemas as api;
use axum::{
  Json,
  extract::{Path, State},
  http::HeaderMap,
};
use gity_core::{AppError, ok};
use serde_json::{Value, json};
type Reply = Result<Json<Value>, AppError>;

/// Registered by the repository router; `routes!` resolves the generated
/// `__path_*` item from this module.
pub(crate) fn routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(refresh))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post, path="/{id}/search/code-index/refresh", tag="Repository",
    summary="Rebuild the code index from the default branch HEAD snapshot",
    params(crate::openapi::schemas::ProjectPath),
    responses((status=200,description="Rebuild the code index from the default branch HEAD snapshot",body=api::Envelope<crate::openapi::repository_schemas::CodeIndexStatus>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn refresh(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  let project = access::project_action(&s, &h, id, ProjectAction::RepositoryAdmin)
    .await?
    .project;
  let status = rebuild(&s, &project).await?;
  Ok(ok(json!({
    "commit":status.commit,
    "files":status.files,
    "definitions":status.definitions,
    "indexed_at":status.indexed_at
  })))
}

async fn rebuild(s: &AppState, project: &Value) -> Result<gity_code_intel::Status, AppError> {
  let repo = crate::repository::repo(project);
  let branch = crate::repository::default_branch(project).to_owned();
  let commit = s
    .git
    .resolve_revision(&repo, &branch, "")
    .await
    .map_err(crate::repository::git_error)?;
  let archive = s
    .git
    .archive(&repo, &commit)
    .await
    .map_err(crate::repository::git_error)?;
  let directory = tempfile::tempdir()
    .map_err(|_| AppError::internal("failed to create a temporary index workspace"))?;
  let project_id = super::project_id(project)?;
  let store = s.code_index.clone();
  super::blocking(move || {
    snapshot::extract(std::io::Cursor::new(archive), directory.path())?;
    let files =
      gity_code_intel::scan::directory(directory.path(), gity_code_intel::Limits::default())?;
    store.refresh(project_id, &gity_code_intel::Archive::new(commit, files))
  })
  .await
}
