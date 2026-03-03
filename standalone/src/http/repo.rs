use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::{
  AccessError, RequiredOrganizationRole, require_organization_role,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use entity::{organization_members, repositories, repository_branches, repository_commits};
use mr_ulid::Ulid;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel, QueryFilter,
  QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListRepositoriesQuery {
  pub organization_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRepositoryRequest {
  pub organization_id: Option<String>,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: Option<String>,
  pub default_branch: Option<String>,
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
  let organization_id = resolve_organization_id(query.organization_id, &current_user)?;
  require_organization_role(
    &state.db_conn,
    current_user.user_id.as_str(),
    organization_id.as_str(),
    RequiredOrganizationRole::Member,
  )
  .await
  .map_err(access_error)?;

  let repositories = repositories::Entity::find()
    .filter(
      Condition::all()
        .add(repositories::Column::OrganizationId.eq(organization_id))
        .add(repositories::Column::DeletedAt.is_null()),
    )
    .all(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to list repositories", err))?;

  let data = repositories.into_iter().map(repository_view).collect();
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
  require_organization_role(
    &state.db_conn,
    current_user.user_id.as_str(),
    organization_id.as_str(),
    RequiredOrganizationRole::Owner,
  )
  .await
  .map_err(access_error)?;

  let exists = repositories::Entity::find()
    .filter(
      Condition::all()
        .add(repositories::Column::OrganizationId.eq(organization_id.clone()))
        .add(repositories::Column::Key.eq(payload.key.clone()))
        .add(repositories::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to check repository key", err))?
    .is_some();

  if exists {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "repository key already exists in this organization".to_string(),
      }),
    ));
  }

  let visibility = parse_visibility(payload.visibility.as_deref()).ok_or_else(|| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "visibility must be private, internal, or public".to_string(),
      }),
    )
  })?;
  let default_branch = payload.default_branch.unwrap_or_else(|| "main".to_string());

  let txn = state
    .db_conn
    .begin()
    .await
    .map_err(|err| internal_error("failed to begin transaction", err))?;

  let repository = repositories::ActiveModel {
    organization_id: Set(organization_id.clone()),
    key: Set(payload.key),
    name: Set(payload.name),
    description: Set(payload.description),
    visibility: Set(visibility),
    default_branch: Set(default_branch.clone()),
    created_by_user_id: Set(current_user.user_id),
    ..Default::default()
  }
  .insert(&txn)
  .await
  .map_err(|err| internal_error("failed to create repository", err))?;

  repository_branches::ActiveModel {
    repository_id: Set(repository.id.clone()),
    name: Set(default_branch),
    is_protected: Set(false),
    last_commit_sha: Set(None),
    ..Default::default()
  }
  .insert(&txn)
  .await
  .map_err(|err| internal_error("failed to create default branch", err))?;

  txn.commit()
    .await
    .map_err(|err| internal_error("failed to commit transaction", err))?;

  Ok((StatusCode::CREATED, Json(repository_view(repository))))
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
  let repository = require_repo_access(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    RequiredOrganizationRole::Member,
  )
  .await?;

  let branches = repository_branches::Entity::find()
    .filter(
      Condition::all()
        .add(repository_branches::Column::RepositoryId.eq(repository.id))
        .add(repository_branches::Column::DeletedAt.is_null()),
    )
    .order_by_asc(repository_branches::Column::Name)
    .all(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to list branches", err))?;

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
  let repository = require_repo_access(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    RequiredOrganizationRole::Member,
  )
  .await?;

  let name = payload.name.trim().to_string();
  if name.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "branch name is required".to_string(),
      }),
    ));
  }

  let exists = repository_branches::Entity::find()
    .filter(
      Condition::all()
        .add(repository_branches::Column::RepositoryId.eq(repository.id.clone()))
        .add(repository_branches::Column::Name.eq(name.clone()))
        .add(repository_branches::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to check branch name", err))?
    .is_some();

  if exists {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "branch already exists".to_string(),
      }),
    ));
  }

  let branch = repository_branches::ActiveModel {
    repository_id: Set(repository.id),
    name: Set(name),
    is_protected: Set(false),
    last_commit_sha: Set(None),
    ..Default::default()
  }
  .insert(&state.db_conn)
  .await
  .map_err(|err| internal_error("failed to create branch", err))?;

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
  let repository = require_repo_access(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    RequiredOrganizationRole::Member,
  )
  .await?;

  let mut finder = repository_commits::Entity::find().filter(
    Condition::all().add(repository_commits::Column::RepositoryId.eq(repository.id)),
  );
  if let Some(branch_name) = query.branch_name {
    finder = finder.filter(repository_commits::Column::BranchName.eq(branch_name));
  }
  let commits = finder
    .order_by_desc(repository_commits::Column::CreatedAt)
    .limit(query.limit.unwrap_or(50))
    .all(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to list commits", err))?;

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
  let (repository, membership) = require_repo_access_with_membership(
    &state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    RequiredOrganizationRole::Member,
  )
  .await?;

  let branch_name = payload.branch_name.trim().to_string();
  if branch_name.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "branch_name is required".to_string(),
      }),
    ));
  }
  let message = payload.message.trim().to_string();
  if message.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "message is required".to_string(),
      }),
    ));
  }

  let branch = repository_branches::Entity::find()
    .filter(
      Condition::all()
        .add(repository_branches::Column::RepositoryId.eq(repository.id.clone()))
        .add(repository_branches::Column::Name.eq(branch_name.clone()))
        .add(repository_branches::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load branch", err))?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "branch not found".to_string(),
        }),
      )
    })?;

  if branch.is_protected && membership.role != organization_members::MemberRole::Owner {
    return Err((
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        message: "only organization owner can commit to protected branch".to_string(),
      }),
    ));
  }

  let commit_sha = payload
    .commit_sha
    .unwrap_or_else(|| Ulid::new().to_string().to_lowercase());

  let exists = repository_commits::Entity::find()
    .filter(
      Condition::all()
        .add(repository_commits::Column::RepositoryId.eq(repository.id.clone()))
        .add(repository_commits::Column::CommitSha.eq(commit_sha.clone())),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to check commit sha", err))?
    .is_some();

  if exists {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "commit sha already exists in this repository".to_string(),
      }),
    ));
  }

  let txn = state
    .db_conn
    .begin()
    .await
    .map_err(|err| internal_error("failed to begin transaction", err))?;

  let commit = repository_commits::ActiveModel {
    repository_id: Set(repository.id.clone()),
    branch_name: Set(branch_name.clone()),
    commit_sha: Set(commit_sha.clone()),
    message: Set(message),
    author_user_id: Set(current_user.user_id),
    ..Default::default()
  }
  .insert(&txn)
  .await
  .map_err(|err| internal_error("failed to insert commit", err))?;

  let mut branch_active = branch.into_active_model();
  branch_active.last_commit_sha = Set(Some(commit_sha));
  branch_active.updated_at = Set(chrono::Utc::now().into());
  branch_active
    .update(&txn)
    .await
    .map_err(|err| internal_error("failed to update branch", err))?;

  txn.commit()
    .await
    .map_err(|err| internal_error("failed to commit transaction", err))?;

  Ok((StatusCode::CREATED, Json(commit_view(commit))))
}

pub fn repo_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_repositories])
    .routes(routes![create_repository])
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
  let repository = require_repo_access(
    state,
    current_user.user_id.as_str(),
    repo_id.as_str(),
    RequiredOrganizationRole::Owner,
  )
  .await?;

  let branch = repository_branches::Entity::find()
    .filter(
      Condition::all()
        .add(repository_branches::Column::RepositoryId.eq(repository.id))
        .add(repository_branches::Column::Name.eq(branch_name))
        .add(repository_branches::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load branch", err))?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "branch not found".to_string(),
        }),
      )
    })?;

  let mut active = branch.into_active_model();
  active.is_protected = Set(is_protected);
  active.updated_at = Set(chrono::Utc::now().into());
  let updated = active
    .update(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to update branch protection", err))?;

  Ok((StatusCode::OK, Json(branch_view(updated))))
}

async fn require_repo_access(
  state: &AppState,
  user_id: &str,
  repo_id: &str,
  required: RequiredOrganizationRole,
) -> Result<repositories::Model, (StatusCode, Json<ErrorResponse>)> {
  let (repo, _) = require_repo_access_with_membership(state, user_id, repo_id, required).await?;
  Ok(repo)
}

async fn require_repo_access_with_membership(
  state: &AppState,
  user_id: &str,
  repo_id: &str,
  required: RequiredOrganizationRole,
) -> Result<(repositories::Model, organization_members::Model), (StatusCode, Json<ErrorResponse>)>
{
  let repository = repositories::Entity::find_by_id(repo_id.to_string())
    .filter(repositories::Column::DeletedAt.is_null())
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load repository", err))?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "repository not found".to_string(),
        }),
      )
    })?;

  let membership = require_organization_role(
    &state.db_conn,
    user_id,
    repository.organization_id.as_str(),
    required,
  )
  .await
  .map_err(access_error)?;

  Ok((repository, membership))
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

fn parse_visibility(value: Option<&str>) -> Option<repositories::RepositoryVisibility> {
  match value.unwrap_or("private").to_ascii_lowercase().as_str() {
    "private" => Some(repositories::RepositoryVisibility::Private),
    "internal" => Some(repositories::RepositoryVisibility::Internal),
    "public" => Some(repositories::RepositoryVisibility::Public),
    _ => None,
  }
}

fn repository_view(model: repositories::Model) -> RepositoryView {
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

fn access_error(err: AccessError) -> (StatusCode, Json<ErrorResponse>) {
  (
    err.status,
    Json(ErrorResponse {
      message: err.message,
    }),
  )
}

fn internal_error(message: &str, err: DbErr) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ErrorResponse {
      message: format!("{message}: {err}"),
    }),
  )
}
