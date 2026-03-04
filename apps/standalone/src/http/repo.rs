use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::security::current_user::CurrentUser;
use crate::service::repository_service::{
  CreateBranchInput, CreateCommitInput, CreateRepositoryInput, ListBranchesInput, ListCommitsInput,
  RepositoryServiceError,
};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use entity::{repositories, repository_branches, repository_commits};
use repository::OrganizationsRepository;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListRepositoriesQuery {
  pub organization_id: Option<String>,
  pub all: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRepositoryRequest {
  pub organization_id: Option<String>,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: Option<String>,
  pub default_branch: Option<String>,
  pub gitignore_template: Option<String>,
  pub license_template: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RepositoryView {
  pub id: String,
  pub organization_id: String,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
  pub clone_http_url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBranchRequest {
  pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchView {
  pub repository_id: String,
  pub name: String,
  pub is_protected: bool,
  pub last_commit_sha: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListCommitsQuery {
  pub branch_name: Option<String>,
  pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCommitRequest {
  pub branch_name: String,
  pub commit_sha: Option<String>,
  pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommitView {
  pub repository_id: String,
  pub branch_name: String,
  pub commit_sha: String,
  pub message: String,
  pub author_user_id: String,
  pub created_at: String,
}

#[utoipa::path(
  get,
  path = "/",
  params(ListRepositoriesQuery),
  responses(
    (status = 200, description = "List repositories in organization", body = [RepositoryView]),
    (status = 400, description = "Missing organization context", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_repositories(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<ListRepositoriesQuery>,
) -> Result<(StatusCode, Json<Vec<RepositoryView>>), (StatusCode, Json<ErrorResponse>)> {
  let repositories = if query.all.unwrap_or(false) {
    state
      .services
      .repository
      .list_repositories_as_super_admin(
        current_user.user_id.as_str(),
        query.organization_id.as_deref(),
      )
      .await
      .map_err(map_repository_service_error)?
  } else {
    let organization_id = resolve_organization_id(query.organization_id, &current_user)?;
    state
      .services
      .repository
      .list_repositories(current_user.user_id.as_str(), organization_id.as_str())
      .await
      .map_err(map_repository_service_error)?
  };

  let organization_ids = repositories
    .iter()
    .map(|repo| repo.organization_id.clone())
    .collect::<Vec<_>>();
  let organizations =
    OrganizationsRepository::list_active_organizations_by_ids(&state.db_conn, organization_ids)
      .await
      .map_err(|err| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(ErrorResponse {
            message: format!("failed to load organizations: {err}"),
          }),
        )
      })?;
  let organization_key_by_id = organizations
    .into_iter()
    .map(|org| (org.id, org.key))
    .collect::<std::collections::HashMap<_, _>>();

  let base_url = public_base_url(&state);
  let data = repositories
    .into_iter()
    .map(|repo| {
      let organization_key = organization_key_by_id
        .get(repo.organization_id.as_str())
        .cloned()
        .unwrap_or_else(|| repo.organization_id.clone());
      repository_view(repo, organization_key.as_str(), base_url.as_str())
    })
    .collect();
  Ok((StatusCode::OK, Json(data)))
}

#[utoipa::path(
  post,
  path = "/",
  request_body = CreateRepositoryRequest,
  responses(
    (status = 201, description = "Repository created", body = RepositoryView),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 409, description = "Repository key already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_repository(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Json(payload): Json<CreateRepositoryRequest>,
) -> Result<(StatusCode, Json<RepositoryView>), (StatusCode, Json<ErrorResponse>)> {
  let organization_id = resolve_organization_id(payload.organization_id, &current_user)?;
  let repository = state
    .services
    .repository
    .create_repository(CreateRepositoryInput {
      organization_id,
      key: payload.key,
      name: payload.name,
      description: payload.description,
      visibility: payload.visibility,
      default_branch: payload.default_branch,
      gitignore_template: payload.gitignore_template,
      license_template: payload.license_template,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  let organization = OrganizationsRepository::find_active_organization_by_id(
    &state.db_conn,
    repository.organization_id.as_str(),
  )
  .await
  .map_err(|err| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        message: format!("failed to load organization: {err}"),
      }),
    )
  })?
  .ok_or_else(|| {
    (
      StatusCode::NOT_FOUND,
      Json(ErrorResponse {
        message: "organization not found".to_string(),
      }),
    )
  })?;

  Ok((
    StatusCode::CREATED,
    Json(repository_view(
      repository,
      organization.key.as_str(),
      public_base_url(&state).as_str(),
    )),
  ))
}

#[utoipa::path(
  delete,
  path = "/{repo_id}",
  responses(
    (status = 204, description = "Repository deleted"),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn delete_repository(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  state
    .services
    .repository
    .delete_repository(current_user.user_id.as_str(), repo_id.as_str())
    .await
    .map_err(map_repository_service_error)?;
  Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
  get,
  path = "/{repo_id}/branches",
  responses(
    (status = 200, description = "List repository branches", body = [BranchView]),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_branches(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<BranchView>>), (StatusCode, Json<ErrorResponse>)> {
  let branches = state
    .services
    .repository
    .list_branches(ListBranchesInput {
      repo_id,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((
    StatusCode::OK,
    Json(branches.into_iter().map(branch_view).collect()),
  ))
}

#[utoipa::path(
  post,
  path = "/{repo_id}/branches",
  request_body = CreateBranchRequest,
  responses(
    (status = 201, description = "Branch created", body = BranchView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository not found", body = ErrorResponse),
    (status = 409, description = "Branch already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_branch(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateBranchRequest>,
) -> Result<(StatusCode, Json<BranchView>), (StatusCode, Json<ErrorResponse>)> {
  let branch = state
    .services
    .repository
    .create_branch(CreateBranchInput {
      repo_id,
      name: payload.name,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::CREATED, Json(branch_view(branch))))
}

#[utoipa::path(
  post,
  path = "/{repo_id}/branches/{branch_name}/protect",
  responses(
    (status = 200, description = "Branch protected", body = BranchView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Branch not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn protect_branch(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((repo_id, branch_name)): Path<(String, String)>,
) -> Result<(StatusCode, Json<BranchView>), (StatusCode, Json<ErrorResponse>)> {
  set_branch_protection(&state, &current_user, repo_id, branch_name, true).await
}

#[utoipa::path(
  post,
  path = "/{repo_id}/branches/{branch_name}/unprotect",
  responses(
    (status = 200, description = "Branch unprotected", body = BranchView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Branch not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn unprotect_branch(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((repo_id, branch_name)): Path<(String, String)>,
) -> Result<(StatusCode, Json<BranchView>), (StatusCode, Json<ErrorResponse>)> {
  set_branch_protection(&state, &current_user, repo_id, branch_name, false).await
}

#[utoipa::path(
  get,
  path = "/{repo_id}/commits",
  params(ListCommitsQuery),
  responses(
    (status = 200, description = "List repository commits", body = [CommitView]),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_commits(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<ListCommitsQuery>,
) -> Result<(StatusCode, Json<Vec<CommitView>>), (StatusCode, Json<ErrorResponse>)> {
  let commits = state
    .services
    .repository
    .list_commits(ListCommitsInput {
      repo_id,
      branch_name: query.branch_name,
      limit: query.limit,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((
    StatusCode::OK,
    Json(commits.into_iter().map(commit_view).collect()),
  ))
}

#[utoipa::path(
  post,
  path = "/{repo_id}/commits",
  request_body = CreateCommitRequest,
  responses(
    (status = 201, description = "Commit recorded", body = CommitView),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Branch not found", body = ErrorResponse),
    (status = 409, description = "Commit already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_commit(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateCommitRequest>,
) -> Result<(StatusCode, Json<CommitView>), (StatusCode, Json<ErrorResponse>)> {
  let commit = state
    .services
    .repository
    .create_commit(CreateCommitInput {
      repo_id,
      branch_name: payload.branch_name,
      commit_sha: payload.commit_sha,
      message: payload.message,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::CREATED, Json(commit_view(commit))))
}

pub fn repo_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_repositories])
    .routes(routes![create_repository])
    .routes(routes![delete_repository])
    .routes(routes![list_branches])
    .routes(routes![create_branch])
    .routes(routes![protect_branch])
    .routes(routes![unprotect_branch])
    .routes(routes![list_commits])
    .routes(routes![create_commit])
}

async fn set_branch_protection(
  state: &AppState,
  current_user: &CurrentUser,
  repo_id: String,
  branch_name: String,
  is_protected: bool,
) -> Result<(StatusCode, Json<BranchView>), (StatusCode, Json<ErrorResponse>)> {
  let branch = state
    .services
    .repository
    .set_branch_protection(
      current_user.user_id.as_str(),
      repo_id.as_str(),
      branch_name.as_str(),
      is_protected,
    )
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::OK, Json(branch_view(branch))))
}

fn resolve_organization_id(
  request_org_id: Option<String>,
  current_user: &CurrentUser,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
  request_org_id
    .or_else(|| current_user.organization_id.clone())
    .ok_or_else(|| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          message: "organization_id is required in request or token".to_string(),
        }),
      )
    })
}

fn repository_view(
  model: repositories::Model,
  organization_key: &str,
  base_url: &str,
) -> RepositoryView {
  let repo_key = model.key.clone();
  RepositoryView {
    id: model.id,
    organization_id: model.organization_id,
    key: model.key,
    name: model.name,
    description: model.description,
    visibility: match model.visibility {
      repositories::RepositoryVisibility::Private => "private".to_string(),
      repositories::RepositoryVisibility::Internal => "internal".to_string(),
      repositories::RepositoryVisibility::Public => "public".to_string(),
    },
    default_branch: model.default_branch,
    clone_http_url: format!("{base_url}/git/{organization_key}/{repo_key}.git"),
  }
}

fn branch_view(model: repository_branches::Model) -> BranchView {
  BranchView {
    repository_id: model.repository_id,
    name: model.name,
    is_protected: model.is_protected,
    last_commit_sha: model.last_commit_sha,
  }
}

fn commit_view(model: repository_commits::Model) -> CommitView {
  CommitView {
    repository_id: model.repository_id,
    branch_name: model.branch_name,
    commit_sha: model.commit_sha,
    message: model.message,
    author_user_id: model.author_user_id,
    created_at: model.created_at.to_rfc3339(),
  }
}

fn map_repository_service_error(err: RepositoryServiceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    RepositoryServiceError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
    RepositoryServiceError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
    RepositoryServiceError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    RepositoryServiceError::Conflict(message) => (StatusCode::CONFLICT, message),
    RepositoryServiceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  };
  (status, Json(ErrorResponse { message }))
}

fn public_base_url(state: &AppState) -> String {
  format!("http://localhost:{}", state.config.server.port)
}
