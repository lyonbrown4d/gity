use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::RequiredOrganizationRole;
use crate::service::repository_service::{
  CreateBranchInput, CreateCommitInput, CreateFileCommitInput, CreateRepositoryInput,
  ListBranchesInput, ListCommitsInput, RepositoryServiceError,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFileCommitRequest {
  pub branch_name: String,
  pub path: String,
  pub content: String,
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

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListRepositoryTreeQuery {
  pub branch_name: Option<String>,
  pub path: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RepositoryTreeEntryView {
  pub name: String,
  pub path: String,
  pub kind: String,
  pub oid: String,
  pub size: Option<usize>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RepositoryBlobQuery {
  pub branch_name: Option<String>,
  pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RepositoryBlobView {
  pub path: String,
  pub content: String,
  pub size: usize,
  pub is_binary: bool,
  pub encoding: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct RepositoryReadmeQuery {
  pub branch_name: Option<String>,
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
  let requested_branch = query.branch_name.clone();
  let resolved_limit = query.limit.unwrap_or(50).clamp(1, 500) as usize;
  let (organization_key, repository_key, resolved_branch) = resolve_repo_storage_context(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    requested_branch.clone(),
  )
  .await?;

  match state
    .services
    .git_backend
    .list_commits(
      organization_key.as_str(),
      repository_key.as_str(),
      resolved_branch.as_str(),
      resolved_limit,
    )
    .await
  {
    Ok(commits) => {
      let data = commits
        .into_iter()
        .map(|commit| CommitView {
          repository_id: repo_id.clone(),
          branch_name: resolved_branch.clone(),
          commit_sha: commit.commit_sha,
          message: commit.message,
          author_user_id: commit.author,
          created_at: unix_seconds_to_rfc3339(commit.authored_at),
        })
        .collect::<Vec<_>>();
      return Ok((StatusCode::OK, Json(data)));
    }
    Err(crate::service::git_backend_service::GitBackendError::StorageNotConfigured) => {}
    Err(err) => return Err(map_repository_content_error(err)),
  }

  let commits = state
    .services
    .repository
    .list_commits(ListCommitsInput {
      repo_id,
      branch_name: requested_branch,
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

#[utoipa::path(
  post,
  path = "/{repo_id}/file-commits",
  request_body = CreateFileCommitRequest,
  responses(
    (status = 201, description = "File committed", body = CommitView),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository or branch not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_file_commit(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateFileCommitRequest>,
) -> Result<(StatusCode, Json<CommitView>), (StatusCode, Json<ErrorResponse>)> {
  let commit = state
    .services
    .repository
    .create_file_commit(CreateFileCommitInput {
      repo_id,
      branch_name: payload.branch_name,
      path: payload.path,
      content: payload.content,
      message: payload.message,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::CREATED, Json(commit_view(commit))))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/tree",
  params(ListRepositoryTreeQuery),
  responses(
    (status = 200, description = "List repository tree entries", body = [RepositoryTreeEntryView]),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository or path not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_repository_tree(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<ListRepositoryTreeQuery>,
) -> Result<(StatusCode, Json<Vec<RepositoryTreeEntryView>>), (StatusCode, Json<ErrorResponse>)> {
  let (organization_key, repository_key, branch_name) = resolve_repo_storage_context(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    query.branch_name,
  )
  .await?;

  let entries = state
    .services
    .git_backend
    .list_tree_entries(
      organization_key.as_str(),
      repository_key.as_str(),
      branch_name.as_str(),
      query.path.as_deref(),
    )
    .await
    .map_err(map_repository_content_error)?;

  Ok((
    StatusCode::OK,
    Json(
      entries
        .into_iter()
        .map(|entry| RepositoryTreeEntryView {
          name: entry.name,
          path: entry.path,
          kind: entry.kind,
          oid: entry.oid,
          size: entry.size,
        })
        .collect(),
    ),
  ))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/blob",
  params(RepositoryBlobQuery),
  responses(
    (status = 200, description = "Read repository blob content", body = RepositoryBlobView),
    (status = 400, description = "Invalid request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Repository or file not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn read_repository_blob(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<RepositoryBlobQuery>,
) -> Result<(StatusCode, Json<RepositoryBlobView>), (StatusCode, Json<ErrorResponse>)> {
  let (organization_key, repository_key, branch_name) = resolve_repo_storage_context(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    query.branch_name,
  )
  .await?;

  let blob = state
    .services
    .git_backend
    .read_blob(
      organization_key.as_str(),
      repository_key.as_str(),
      branch_name.as_str(),
      query.path.as_str(),
    )
    .await
    .map_err(map_repository_content_error)?;
  let (content, is_binary, encoding) = decode_blob_content(blob.content.as_slice());

  Ok((
    StatusCode::OK,
    Json(RepositoryBlobView {
      path: blob.path,
      content,
      size: blob.size,
      is_binary,
      encoding,
    }),
  ))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/readme",
  params(RepositoryReadmeQuery),
  responses(
    (status = 200, description = "Read repository root README", body = RepositoryBlobView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "README not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn read_repository_readme(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<RepositoryReadmeQuery>,
) -> Result<(StatusCode, Json<RepositoryBlobView>), (StatusCode, Json<ErrorResponse>)> {
  let (organization_key, repository_key, branch_name) = resolve_repo_storage_context(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    query.branch_name,
  )
  .await?;

  let blob = state
    .services
    .git_backend
    .read_root_readme(
      organization_key.as_str(),
      repository_key.as_str(),
      branch_name.as_str(),
    )
    .await
    .map_err(map_repository_content_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "README not found".to_string(),
        }),
      )
    })?;

  let (content, is_binary, encoding) = decode_blob_content(blob.content.as_slice());
  Ok((
    StatusCode::OK,
    Json(RepositoryBlobView {
      path: blob.path,
      content,
      size: blob.size,
      is_binary,
      encoding,
    }),
  ))
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
    .routes(routes![create_file_commit])
    .routes(routes![list_repository_tree])
    .routes(routes![read_repository_blob])
    .routes(routes![read_repository_readme])
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
    clone_http_url: format!("{base_url}/{organization_key}/{repo_key}.git"),
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

async fn resolve_repo_storage_context(
  state: &AppState,
  user_id: &str,
  repo_id: &str,
  branch_name: Option<String>,
) -> Result<(String, String, String), (StatusCode, Json<ErrorResponse>)> {
  let repository = state
    .services
    .repository
    .require_repo_access(user_id, repo_id, RequiredOrganizationRole::Member)
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

  let branch = branch_name.unwrap_or(repository.default_branch.clone());
  Ok((organization.key, repository.key, branch))
}

fn decode_blob_content(bytes: &[u8]) -> (String, bool, String) {
  match std::str::from_utf8(bytes) {
    Ok(value) => (value.to_string(), false, "utf-8".to_string()),
    Err(_) => (
      String::from_utf8_lossy(bytes).to_string(),
      true,
      "utf-8-lossy".to_string(),
    ),
  }
}

fn unix_seconds_to_rfc3339(seconds: i64) -> String {
  chrono::DateTime::from_timestamp(seconds, 0)
    .map(|value| value.to_rfc3339())
    .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::UNIX_EPOCH.to_rfc3339())
}

fn map_repository_content_error(
  err: crate::service::git_backend_service::GitBackendError,
) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    crate::service::git_backend_service::GitBackendError::InvalidRepositoryPath => {
      (StatusCode::BAD_REQUEST, "invalid repository path".to_string())
    }
    crate::service::git_backend_service::GitBackendError::RepositoryNotFound => {
      (StatusCode::NOT_FOUND, "repository not found".to_string())
    }
    crate::service::git_backend_service::GitBackendError::InvalidComponent(message) => {
      (StatusCode::BAD_REQUEST, message)
    }
    crate::service::git_backend_service::GitBackendError::Git(message) => {
      let normalized = message.to_ascii_lowercase();
      if normalized.contains("not found") {
        (StatusCode::NOT_FOUND, message)
      } else if normalized.contains("invalid path")
        || normalized.contains("not a directory")
        || normalized.contains("not a file")
      {
        (StatusCode::BAD_REQUEST, message)
      } else {
        (StatusCode::INTERNAL_SERVER_ERROR, message)
      }
    }
    crate::service::git_backend_service::GitBackendError::StorageNotConfigured
    | crate::service::git_backend_service::GitBackendError::AlreadyExists(_)
    | crate::service::git_backend_service::GitBackendError::Io(_)
    | crate::service::git_backend_service::GitBackendError::Db(_)
    | crate::service::git_backend_service::GitBackendError::Utf8(_) => {
      (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
  };
  (status, Json(ErrorResponse { message }))
}
