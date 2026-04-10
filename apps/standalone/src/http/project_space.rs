use crate::http::app_state::AppState;
use crate::service::project_space_service::ProjectSpaceError;
use application::namespace::{CreateNamespaceCommand, NamespaceView};
use application::project::{CreateProjectCommand, ProjectView};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
  pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateNamespaceRequest {
  pub parent_namespace_id: Option<i64>,
  pub owner_user_id: Option<String>,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub kind: String,
  pub visibility: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
  pub namespace_id: i64,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: Option<String>,
  pub actor_user_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NamespaceResponse {
  pub id: i64,
  pub full_path: String,
  pub parent_namespace_id: Option<i64>,
  pub owner_user_id: Option<String>,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub kind: String,
  pub visibility: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectResponse {
  pub id: i64,
  pub namespace_id: i64,
  pub full_path: String,
  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
  pub archived: bool,
  pub created_by_user_id: String,
}

#[utoipa::path(
  post,
  path = "/",
  tag = "Namespaces",
  request_body = CreateNamespaceRequest,
  responses(
    (status = 201, description = "Namespace created", body = NamespaceResponse),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 404, description = "Parent namespace not found", body = ErrorResponse),
    (status = 409, description = "Namespace path already exists", body = ErrorResponse)
  )
)]
pub async fn create_namespace(
  State(state): State<AppState>,
  Json(payload): Json<CreateNamespaceRequest>,
) -> Result<(StatusCode, Json<NamespaceResponse>), (StatusCode, Json<ErrorResponse>)> {
  let namespace = state
    .project_space
    .create_namespace(CreateNamespaceCommand {
      parent_namespace_id: payload.parent_namespace_id,
      owner_user_id: payload.owner_user_id,
      path_key: payload.path_key,
      name: payload.name,
      description: payload.description,
      kind: payload.kind,
      visibility: payload.visibility,
    })
    .await
    .map_err(map_error)?;

  Ok((StatusCode::CREATED, Json(namespace.into())))
}

#[utoipa::path(
  get,
  path = "/",
  tag = "Namespaces",
  responses((status = 200, description = "Namespaces listed", body = Vec<NamespaceResponse>))
)]
pub async fn list_namespaces(
  State(state): State<AppState>,
) -> Result<Json<Vec<NamespaceResponse>>, (StatusCode, Json<ErrorResponse>)> {
  let items = state
    .project_space
    .list_namespaces()
    .await
    .map_err(map_error)?;
  Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
  get,
  path = "/{namespace_id}",
  tag = "Namespaces",
  params(("namespace_id" = i64, Path, description = "Namespace id")),
  responses(
    (status = 200, description = "Namespace found", body = NamespaceResponse),
    (status = 404, description = "Namespace not found", body = ErrorResponse)
  )
)]
pub async fn get_namespace(
  State(state): State<AppState>,
  Path(namespace_id): Path<i64>,
) -> Result<Json<NamespaceResponse>, (StatusCode, Json<ErrorResponse>)> {
  let namespace = state
    .project_space
    .get_namespace(namespace_id)
    .await
    .map_err(map_error)?
    .ok_or_else(|| not_found("namespace not found"))?;
  Ok(Json(namespace.into()))
}

#[utoipa::path(
  post,
  path = "/",
  tag = "Projects",
  request_body = CreateProjectRequest,
  responses(
    (status = 201, description = "Project created", body = ProjectResponse),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 404, description = "Namespace not found", body = ErrorResponse),
    (status = 409, description = "Project path already exists", body = ErrorResponse)
  )
)]
pub async fn create_project(
  State(state): State<AppState>,
  Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, Json<ErrorResponse>)> {
  let project = state
    .project_space
    .create_project(CreateProjectCommand {
      namespace_id: payload.namespace_id,
      path_key: payload.path_key,
      name: payload.name,
      description: payload.description,
      visibility: payload.visibility,
      default_branch: payload.default_branch,
      actor_user_id: payload.actor_user_id,
    })
    .await
    .map_err(map_error)?;

  if let Some(job_client) = state.repository_language_jobs.as_ref()
    && let Err(err) = job_client
      .enqueue_project_branch(project.id, Some(project.default_branch.as_str()))
      .await
  {
    warn!(
      project_id = project.id,
      branch = project.default_branch.as_str(),
      error = err,
      "failed to enqueue initial project language job"
    );
  }

  Ok((StatusCode::CREATED, Json(project.into())))
}

#[utoipa::path(
  get,
  path = "/",
  tag = "Projects",
  responses((status = 200, description = "Projects listed", body = Vec<ProjectResponse>))
)]
pub async fn list_projects(
  State(state): State<AppState>,
) -> Result<Json<Vec<ProjectResponse>>, (StatusCode, Json<ErrorResponse>)> {
  let items = state
    .project_space
    .list_projects()
    .await
    .map_err(map_error)?;
  Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
  get,
  path = "/{project_id}",
  tag = "Projects",
  params(("project_id" = i64, Path, description = "Project id")),
  responses(
    (status = 200, description = "Project found", body = ProjectResponse),
    (status = 404, description = "Project not found", body = ErrorResponse)
  )
)]
pub async fn get_project(
  State(state): State<AppState>,
  Path(project_id): Path<i64>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
  let project = state
    .project_space
    .get_project(project_id)
    .await
    .map_err(map_error)?
    .ok_or_else(|| not_found("project not found"))?;
  Ok(Json(project.into()))
}

pub fn namespace_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![create_namespace])
    .routes(routes![list_namespaces])
    .routes(routes![get_namespace])
}

pub fn project_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![create_project])
    .routes(routes![list_projects])
    .routes(routes![get_project])
}

impl From<NamespaceView> for NamespaceResponse {
  fn from(value: NamespaceView) -> Self {
    Self {
      id: value.id,
      full_path: value.full_path,
      parent_namespace_id: value.parent_namespace_id,
      owner_user_id: value.owner_user_id,
      path_key: value.path_key,
      name: value.name,
      description: value.description,
      kind: value.kind,
      visibility: value.visibility,
    }
  }
}

impl From<ProjectView> for ProjectResponse {
  fn from(value: ProjectView) -> Self {
    Self {
      id: value.id,
      namespace_id: value.namespace_id,
      full_path: value.full_path,
      path_key: value.path_key,
      name: value.name,
      description: value.description,
      visibility: value.visibility,
      default_branch: value.default_branch,
      archived: value.archived,
      created_by_user_id: value.created_by_user_id,
    }
  }
}

fn map_error(err: ProjectSpaceError) -> (StatusCode, Json<ErrorResponse>) {
  match err {
    ProjectSpaceError::BadRequest(message) => {
      (StatusCode::BAD_REQUEST, Json(ErrorResponse { message }))
    }
    ProjectSpaceError::NotFound(message) => {
      (StatusCode::NOT_FOUND, Json(ErrorResponse { message }))
    }
    ProjectSpaceError::Conflict(message) => (StatusCode::CONFLICT, Json(ErrorResponse { message })),
    ProjectSpaceError::Forbidden(message) => {
      (StatusCode::FORBIDDEN, Json(ErrorResponse { message }))
    }
    ProjectSpaceError::Internal(message) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse { message }),
    ),
  }
}

fn not_found(message: &str) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::NOT_FOUND,
    Json(ErrorResponse {
      message: message.to_string(),
    }),
  )
}
