use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::http::pagination::{resolve_pagination, to_page, to_single_page};
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::RequiredOrganizationRole;
use crate::service::project_space_service::ProjectSpaceError;
use crate::service::repository_service::{
  CreateBranchInput, CreateCommitInput, CreateFileCommitInput, CreateIssueCommentInput,
  CreateIssueInput, CreateRepositoryInput, GetIssueByNumberInput, ListBranchesInput,
  ListCommitsInput, ListIssueCommentsInput, ListIssuesInput, RepositoryServiceError,
  UpdateIssueInput, UploadIssueAttachmentInput, UploadIssueAttachmentOutput,
};
use application::project::{
  CreateProjectBranchCommand, CreateProjectCommand, CreateProjectIssueCommand,
  CreateProjectIssueCommentCommand, ProjectBranchView as ProjectSpaceBranchView,
  ProjectIssueCommentView as ProjectSpaceIssueCommentView,
  ProjectIssueView as ProjectSpaceIssueView,
  ProjectLanguageSnapshotView as ProjectSpaceLanguageSnapshotView,
  SetProjectBranchProtectionCommand, UpdateProjectIssueCommand,
};
use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use domain::api::crud::parse_csv_ids;
use domain::api::repository::{CreateRepositoryRequest, ListRepositoriesQuery, RepositoryView};
use domain::api::response::{ApiResponse, EmptyData};
use domain::page::Page;
use entity::{
  repositories, repository_branches, repository_commits, repository_issue_comments,
  repository_issues,
};
use repository::{OrganizationsRepository, RepositoryLanguageSnapshotsRepository};
use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateIssueRequest {
  pub title: String,
  pub description: Option<String>,
  pub assignee_user_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateIssueRequest {
  pub title: Option<String>,
  #[serde(default)]
  pub description: Option<Option<String>>,
  pub status: Option<String>,
  #[serde(default)]
  pub assignee_user_id: Option<Option<String>>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListIssuesQuery {
  pub status: Option<String>,
  pub limit: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IssueView {
  pub id: String,
  pub repository_id: String,
  pub number: i32,
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub author_user_id: String,
  pub assignee_user_id: Option<String>,
  pub created_at: String,
  pub updated_at: String,
  pub closed_at: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateIssueCommentRequest {
  pub content: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListIssueCommentsQuery {
  pub limit: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IssueCommentView {
  pub id: String,
  pub issue_id: String,
  pub author_user_id: String,
  pub content: String,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct UploadIssueAttachmentQuery {
  pub issue_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IssueAttachmentUploadView {
  pub url: String,
  pub object_key: String,
  pub file_name: String,
  pub content_type: String,
  pub size: usize,
  pub markdown: String,
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

#[derive(Debug, Deserialize, IntoParams)]
pub struct RepositoryLanguagesQuery {
  pub branch_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RepositoryLanguageItemView {
  pub language: String,
  pub bytes: u64,
  pub percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RepositoryLanguagesView {
  pub repository_id: String,
  pub branch_name: String,
  pub status: String,
  pub revision: Option<String>,
  pub analyzed_at: Option<String>,
  pub total_bytes: u64,
  pub languages: Vec<RepositoryLanguageItemView>,
}

#[utoipa::path(
  get,
  path = "/",
  tag = "Repositories",
  params(ListRepositoriesQuery),
  responses(
    (status = 200, description = "List repositories in organization", body = ApiResponse<Page<RepositoryView>>),
    (status = 400, description = "Missing organization context", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_repositories(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<ListRepositoriesQuery>,
) -> Result<(StatusCode, Json<Page<RepositoryView>>), (StatusCode, Json<ErrorResponse>)> {
  if should_use_project_space_repo_api(
    query.organization_id.as_deref(),
    &current_user,
    query.all.unwrap_or(false),
  ) {
    return list_project_repositories(state, current_user, query).await;
  }

  let ids = parse_csv_ids(query.ids.as_deref());
  let has_ids_filter = !ids.is_empty();
  let ids_filter = ids.into_iter().collect::<std::collections::HashSet<_>>();
  let pagination = if has_ids_filter {
    None
  } else {
    Some(resolve_pagination(query.page, query.page_size, 50, 200)?)
  };

  let (repositories, total, use_single_page) = if query.all.unwrap_or(false) {
    if has_ids_filter {
      let repositories = state
        .services
        .repository
        .list_repositories_as_super_admin(
          current_user.user_id.as_str(),
          query.organization_id.as_deref(),
        )
        .await
        .map_err(map_repository_service_error)?;
      let filtered = repositories
        .into_iter()
        .filter(|repo| ids_filter.contains(&repo.id))
        .collect::<Vec<_>>();
      let total = filtered.len() as u64;
      (filtered, total, true)
    } else {
      let pagination = pagination.ok_or_else(|| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(ErrorResponse {
            message: "pagination state is invalid".to_string(),
          }),
        )
      })?;
      let (repositories, total) = state
        .services
        .repository
        .list_repositories_as_super_admin_page(
          current_user.user_id.as_str(),
          query.organization_id.as_deref(),
          pagination.page,
          pagination.page_size,
        )
        .await
        .map_err(map_repository_service_error)?;
      (repositories, total, false)
    }
  } else {
    let organization_id = resolve_organization_id(query.organization_id, &current_user)?;
    if has_ids_filter {
      let repositories = state
        .services
        .repository
        .list_repositories(current_user.user_id.as_str(), organization_id.as_str())
        .await
        .map_err(map_repository_service_error)?;
      let filtered = repositories
        .into_iter()
        .filter(|repo| ids_filter.contains(&repo.id))
        .collect::<Vec<_>>();
      let total = filtered.len() as u64;
      (filtered, total, true)
    } else {
      let pagination = pagination.ok_or_else(|| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(ErrorResponse {
            message: "pagination state is invalid".to_string(),
          }),
        )
      })?;
      let (repositories, total) = state
        .services
        .repository
        .list_repositories_page(
          current_user.user_id.as_str(),
          organization_id.as_str(),
          pagination.page,
          pagination.page_size,
        )
        .await
        .map_err(map_repository_service_error)?;
      (repositories, total, false)
    }
  };

  let organization_ids = repositories
    .iter()
    .map(|repo| repo.organization_id.clone())
    .collect::<Vec<_>>();
  let organizations = OrganizationsRepository::new(state.db_conn.clone())
    .list_active_organizations_by_ids(organization_ids)
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
    .collect::<Vec<_>>();

  if use_single_page {
    return Ok((StatusCode::OK, Json(to_single_page(data))));
  }

  let pagination = pagination.ok_or_else(|| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        message: "pagination state is invalid".to_string(),
      }),
    )
  })?;
  Ok((StatusCode::OK, Json(to_page(data, total, pagination))))
}
#[utoipa::path(
  get,
  path = "/{repo_id}",
  tag = "Repositories",
  responses(
    (status = 200, description = "Get repository by id", body = ApiResponse<RepositoryView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn get_repository(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
) -> Result<(StatusCode, Json<RepositoryView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let view = get_project_repository_view(&state, &current_user, project_id).await?;
    return Ok((StatusCode::OK, Json(view)));
  }

  let repository = state
    .services
    .repository
    .require_repo_access(
      current_user.user_id.as_str(),
      repo_id.as_str(),
      RequiredOrganizationRole::Member,
    )
    .await
    .map_err(map_repository_service_error)?;

  let organization = OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_id(repository.organization_id.as_str())
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

  let view = repository_view(
    repository,
    organization.key.as_str(),
    public_base_url(&state).as_str(),
  );
  Ok((StatusCode::OK, Json(view)))
}
#[utoipa::path(
  post,
  path = "/",
  tag = "Repositories",
  request_body = CreateRepositoryRequest,
  responses(
    (status = 201, description = "Repository created", body = ApiResponse<RepositoryView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Repository key already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn create_repository(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Json(payload): Json<CreateRepositoryRequest>,
) -> Result<(StatusCode, Json<RepositoryView>), (StatusCode, Json<ErrorResponse>)> {
  let organization_id = resolve_organization_id(payload.organization_id.clone(), &current_user)?;
  if let Ok(namespace_id) = organization_id.parse::<i64>() {
    let view = create_project_repository(&state, &current_user, namespace_id, payload).await?;
    return Ok((StatusCode::CREATED, Json(view)));
  }

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

  let organization = OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_id(repository.organization_id.as_str())
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
  enqueue_repository_language_job(
    &state,
    repository.id.as_str(),
    Some(repository.default_branch.as_str()),
  )
  .await;

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
  tag = "Repositories",
  responses(
    (status = 200, description = "Repository deleted", body = ApiResponse<EmptyData>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn delete_repository(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
) -> Result<(StatusCode, Json<EmptyData>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let project = state
      .project_space
      .get_project(project_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "repository not found".to_string(),
          }),
        )
      })?;
    let namespace = state
      .project_space
      .get_namespace(project.namespace_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "organization not found".to_string(),
          }),
        )
      })?;

    if !current_user_is_repo_super_admin(&state, current_user.user_id.as_str()).await? {
      let allowed = state
        .project_space
        .user_has_namespace_role(
          namespace.id,
          current_user.user_id.as_str(),
          &["owner", "maintainer"],
        )
        .await
        .map_err(map_project_space_error)?;
      let owns_namespace =
        namespace.owner_user_id.as_deref() == Some(current_user.user_id.as_str());
      if !allowed && !owns_namespace {
        return Err((
          StatusCode::FORBIDDEN,
          Json(ErrorResponse {
            message: "organization owner or maintainer permission is required".to_string(),
          }),
        ));
      }
    }

    state
      .project_space
      .delete_project(project_id)
      .await
      .map_err(map_project_space_error)?;
    return Ok((StatusCode::OK, Json(EmptyData {})));
  }

  state
    .services
    .repository
    .delete_repository(current_user.user_id.as_str(), repo_id.as_str())
    .await
    .map_err(map_repository_service_error)?;
  Ok((StatusCode::OK, Json(EmptyData {})))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/branches",
  tag = "Repositories",
  responses(
    (status = 200, description = "List repository branches", body = ApiResponse<Vec<BranchView>>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_branches(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<BranchView>>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "you are not a member of this organization",
    )
    .await?;
    let branches = state
      .project_space
      .list_project_branches(project_id)
      .await
      .map_err(map_project_space_error)?;
    return Ok((
      StatusCode::OK,
      Json(
        branches
          .into_iter()
          .map(|branch| project_branch_api_view(repo_id.as_str(), branch))
          .collect::<Vec<_>>(),
      ),
    ));
  }

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
  tag = "Repositories",
  request_body = CreateBranchRequest,
  responses(
    (status = 201, description = "Branch created", body = ApiResponse<BranchView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Branch already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn create_branch(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateBranchRequest>,
) -> Result<(StatusCode, Json<BranchView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      Some(&["developer", "maintainer", "owner"]),
      "organization developer permission is required",
    )
    .await?;
    let branch = state
      .project_space
      .create_project_branch(CreateProjectBranchCommand {
        project_id,
        name: payload.name,
        source_branch: None,
      })
      .await
      .map_err(map_project_space_error)?;
    return Ok((
      StatusCode::CREATED,
      Json(project_branch_api_view(repo_id.as_str(), branch)),
    ));
  }

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
  tag = "Repositories",
  responses(
    (status = 200, description = "Branch protected", body = ApiResponse<BranchView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Branch not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Repositories",
  responses(
    (status = 200, description = "Branch unprotected", body = ApiResponse<BranchView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Branch not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Repositories",
  params(ListCommitsQuery),
  responses(
    (status = 200, description = "List repository commits", body = ApiResponse<Vec<CommitView>>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Repositories",
  request_body = CreateCommitRequest,
  responses(
    (status = 201, description = "Commit recorded", body = ApiResponse<CommitView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Branch not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Commit already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn create_commit(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateCommitRequest>,
) -> Result<(StatusCode, Json<CommitView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let (project, namespace) = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      Some(&["developer", "maintainer", "owner"]),
      "organization developer permission is required",
    )
    .await?;

    if payload
      .commit_sha
      .as_deref()
      .is_some_and(|value| !value.trim().is_empty())
    {
      return Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          message: "commit_sha is generated by git for project-backed repositories".to_string(),
        }),
      ));
    }

    if let Some(branch) = state
      .project_space
      .get_project_branch(project.id, payload.branch_name.as_str())
      .await
      .map_err(map_project_space_error)?
      && branch.is_protected
      && namespace.owner_user_id.as_deref() != Some(current_user.user_id.as_str())
    {
      return Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
          message: "only organization owner can commit to protected branch".to_string(),
        }),
      ));
    }

    let author = state
      .services
      .user
      .get_current_user(current_user.user_id.as_str())
      .await
      .map_err(map_user_service_error_as_repo_error)?;
    let commit_sha = state
      .services
      .git_backend
      .create_empty_commit(
        namespace.full_path.as_str(),
        project.path_key.as_str(),
        payload.branch_name.as_str(),
        payload.message.as_str(),
        author.username.as_str(),
        author.email.as_str(),
      )
      .await
      .map_err(map_repository_content_error)?;

    let _ = state
      .project_space
      .list_project_branches(project.id)
      .await
      .map_err(map_project_space_error)?;
    enqueue_repository_language_job(&state, repo_id.as_str(), Some(payload.branch_name.as_str()))
      .await;

    return Ok((
      StatusCode::CREATED,
      Json(CommitView {
        repository_id: repo_id,
        branch_name: payload.branch_name,
        commit_sha,
        message: payload.message,
        author_user_id: current_user.user_id,
        created_at: chrono::Utc::now().to_rfc3339(),
      }),
    ));
  }

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
  tag = "Repositories",
  request_body = CreateFileCommitRequest,
  responses(
    (status = 201, description = "File committed", body = ApiResponse<CommitView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository or branch not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn create_file_commit(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateFileCommitRequest>,
) -> Result<(StatusCode, Json<CommitView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let project = state
      .project_space
      .get_project(project_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "repository not found".to_string(),
          }),
        )
      })?;
    let namespace = state
      .project_space
      .get_namespace(project.namespace_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "organization not found".to_string(),
          }),
        )
      })?;

    if !current_user_is_repo_super_admin(&state, current_user.user_id.as_str()).await? {
      let allowed = state
        .project_space
        .user_has_namespace_role(
          namespace.id,
          current_user.user_id.as_str(),
          &["developer", "maintainer", "owner"],
        )
        .await
        .map_err(map_project_space_error)?;
      let owns_namespace =
        namespace.owner_user_id.as_deref() == Some(current_user.user_id.as_str());
      if !allowed && !owns_namespace {
        return Err((
          StatusCode::FORBIDDEN,
          Json(ErrorResponse {
            message: "organization developer permission is required".to_string(),
          }),
        ));
      }
    }

    if let Some(branch) = state
      .project_space
      .get_project_branch(project.id, payload.branch_name.as_str())
      .await
      .map_err(map_project_space_error)?
      && branch.is_protected
      && namespace.owner_user_id.as_deref() != Some(current_user.user_id.as_str())
    {
      return Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
          message: "only organization owner can commit to protected branch".to_string(),
        }),
      ));
    }

    let author = state
      .services
      .user
      .get_current_user(current_user.user_id.as_str())
      .await
      .map_err(map_user_service_error_as_repo_error)?;
    let commit_sha = state
      .services
      .git_backend
      .commit_file(
        namespace.full_path.as_str(),
        project.path_key.as_str(),
        payload.branch_name.as_str(),
        payload.path.as_str(),
        payload.content.as_str(),
        payload.message.as_str(),
        author.username.as_str(),
        author.email.as_str(),
      )
      .await
      .map_err(map_repository_content_error)?;

    enqueue_repository_language_job(&state, repo_id.as_str(), Some(payload.branch_name.as_str()))
      .await;

    return Ok((
      StatusCode::CREATED,
      Json(CommitView {
        repository_id: repo_id,
        branch_name: payload.branch_name,
        commit_sha,
        message: payload.message,
        author_user_id: current_user.user_id,
        created_at: chrono::Utc::now().to_rfc3339(),
      }),
    ));
  }

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
  enqueue_repository_language_job(
    &state,
    commit.repository_id.as_str(),
    Some(commit.branch_name.as_str()),
  )
  .await;

  Ok((StatusCode::CREATED, Json(commit_view(commit))))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/issues",
  tag = "Repositories",
  params(ListIssuesQuery),
  responses(
    (status = 200, description = "List repository issues", body = ApiResponse<Vec<IssueView>>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_issues(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<ListIssuesQuery>,
) -> Result<(StatusCode, Json<Vec<IssueView>>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "you are not a member of this organization",
    )
    .await?;
    let issues = state
      .project_space
      .list_project_issues(project_id, query.status.as_deref(), query.limit)
      .await
      .map_err(map_project_space_error)?;
    return Ok((
      StatusCode::OK,
      Json(
        issues
          .into_iter()
          .map(project_issue_api_view)
          .collect::<Vec<_>>(),
      ),
    ));
  }

  let issues = state
    .services
    .repository
    .list_issues(ListIssuesInput {
      repo_id,
      status: query.status,
      limit: query.limit,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((
    StatusCode::OK,
    Json(issues.into_iter().map(issue_view).collect()),
  ))
}

#[utoipa::path(
  post,
  path = "/{repo_id}/issues",
  tag = "Repositories",
  request_body = CreateIssueRequest,
  responses(
    (status = 201, description = "Issue created", body = ApiResponse<IssueView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn create_issue(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Json(payload): Json<CreateIssueRequest>,
) -> Result<(StatusCode, Json<IssueView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "organization member permission is required",
    )
    .await?;
    let issue = state
      .project_space
      .create_project_issue(CreateProjectIssueCommand {
        project_id,
        title: payload.title,
        description: payload.description,
        assignee_user_id: payload.assignee_user_id,
        author_user_id: current_user.user_id,
      })
      .await
      .map_err(map_project_space_error)?;
    return Ok((StatusCode::CREATED, Json(project_issue_api_view(issue))));
  }

  let issue = state
    .services
    .repository
    .create_issue(CreateIssueInput {
      repo_id,
      title: payload.title,
      description: payload.description,
      assignee_user_id: payload.assignee_user_id,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::CREATED, Json(issue_view(issue))))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/issues/by-number/{number}",
  tag = "Repositories",
  responses(
    (status = 200, description = "Get issue by number", body = ApiResponse<IssueView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Issue not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn get_issue_by_number(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((repo_id, number)): Path<(String, i32)>,
) -> Result<(StatusCode, Json<IssueView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "you are not a member of this organization",
    )
    .await?;
    let issue = state
      .project_space
      .get_project_issue_by_iid(project_id, number as i64)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "issue not found".to_string(),
          }),
        )
      })?;
    return Ok((StatusCode::OK, Json(project_issue_api_view(issue))));
  }

  let issue = state
    .services
    .repository
    .get_issue_by_number(GetIssueByNumberInput {
      repo_id,
      number,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::OK, Json(issue_view(issue))))
}

#[utoipa::path(
  patch,
  path = "/{repo_id}/issues/{issue_id}",
  tag = "Repositories",
  request_body = UpdateIssueRequest,
  responses(
    (status = 200, description = "Issue updated", body = ApiResponse<IssueView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Issue not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn update_issue(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((repo_id, issue_id)): Path<(String, String)>,
  Json(payload): Json<UpdateIssueRequest>,
) -> Result<(StatusCode, Json<IssueView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let (_, namespace) = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "you are not a member of this organization",
    )
    .await?;
    let issue_id = parse_project_issue_id(issue_id.as_str())?;
    let existing = state
      .project_space
      .get_project_issue(project_id, issue_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "issue not found".to_string(),
          }),
        )
      })?;

    if !current_user_is_repo_super_admin(&state, current_user.user_id.as_str()).await? {
      let can_manage = state
        .project_space
        .user_has_namespace_role(
          namespace.id,
          current_user.user_id.as_str(),
          &["maintainer", "owner"],
        )
        .await
        .map_err(map_project_space_error)?;
      if !can_manage && existing.author_user_id != current_user.user_id {
        return Err((
          StatusCode::FORBIDDEN,
          Json(ErrorResponse {
            message: "issue author or organization maintainer permission is required".to_string(),
          }),
        ));
      }
    }

    let issue = state
      .project_space
      .update_project_issue(UpdateProjectIssueCommand {
        project_id,
        issue_id,
        title: payload.title,
        description: payload.description,
        state: payload.status,
        assignee_user_id: payload.assignee_user_id,
      })
      .await
      .map_err(map_project_space_error)?;
    return Ok((StatusCode::OK, Json(project_issue_api_view(issue))));
  }

  let issue = state
    .services
    .repository
    .update_issue(UpdateIssueInput {
      repo_id,
      issue_id,
      title: payload.title,
      description: payload.description,
      status: payload.status,
      assignee_user_id: payload.assignee_user_id,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::OK, Json(issue_view(issue))))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/issues/{issue_id}/comments",
  tag = "Repositories",
  params(ListIssueCommentsQuery),
  responses(
    (status = 200, description = "List issue comments", body = ApiResponse<Vec<IssueCommentView>>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Issue not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_issue_comments(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((repo_id, issue_id)): Path<(String, String)>,
  Query(query): Query<ListIssueCommentsQuery>,
) -> Result<(StatusCode, Json<Vec<IssueCommentView>>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "you are not a member of this organization",
    )
    .await?;
    let issue_id = parse_project_issue_id(issue_id.as_str())?;
    let comments = state
      .project_space
      .list_project_issue_comments(project_id, issue_id, query.limit)
      .await
      .map_err(map_project_space_error)?;
    return Ok((
      StatusCode::OK,
      Json(
        comments
          .into_iter()
          .map(project_issue_comment_api_view)
          .collect::<Vec<_>>(),
      ),
    ));
  }

  let comments = state
    .services
    .repository
    .list_issue_comments(ListIssueCommentsInput {
      repo_id,
      issue_id,
      limit: query.limit,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((
    StatusCode::OK,
    Json(comments.into_iter().map(issue_comment_view).collect()),
  ))
}

#[utoipa::path(
  post,
  path = "/{repo_id}/issues/{issue_id}/comments",
  tag = "Repositories",
  request_body = CreateIssueCommentRequest,
  responses(
    (status = 201, description = "Issue comment created", body = ApiResponse<IssueCommentView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Issue not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn create_issue_comment(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((repo_id, issue_id)): Path<(String, String)>,
  Json(payload): Json<CreateIssueCommentRequest>,
) -> Result<(StatusCode, Json<IssueCommentView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "organization member permission is required",
    )
    .await?;
    let issue_id = parse_project_issue_id(issue_id.as_str())?;
    let comment = state
      .project_space
      .create_project_issue_comment(CreateProjectIssueCommentCommand {
        project_id,
        issue_id,
        body: payload.content,
        author_user_id: current_user.user_id,
      })
      .await
      .map_err(map_project_space_error)?;
    return Ok((
      StatusCode::CREATED,
      Json(project_issue_comment_api_view(comment)),
    ));
  }

  let comment = state
    .services
    .repository
    .create_issue_comment(CreateIssueCommentInput {
      repo_id,
      issue_id,
      content: payload.content,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((StatusCode::CREATED, Json(issue_comment_view(comment))))
}

#[utoipa::path(
  post,
  path = "/{repo_id}/issues/attachments",
  tag = "Repositories",
  params(UploadIssueAttachmentQuery),
  responses(
    (status = 201, description = "Issue attachment uploaded", body = ApiResponse<IssueAttachmentUploadView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Issue not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn upload_issue_attachment(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<UploadIssueAttachmentQuery>,
  mut multipart: Multipart,
) -> Result<(StatusCode, Json<IssueAttachmentUploadView>), (StatusCode, Json<ErrorResponse>)> {
  let mut file_name: Option<String> = None;
  let mut content_type: Option<String> = None;
  let mut bytes: Option<Vec<u8>> = None;

  while let Some(field) = multipart.next_field().await.map_err(|err| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: format!("failed to parse multipart form: {err}"),
      }),
    )
  })? {
    if field.name() != Some("file") {
      continue;
    }
    file_name = field.file_name().map(ToString::to_string);
    content_type = field.content_type().map(ToString::to_string);
    let body = field.bytes().await.map_err(|err| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          message: format!("failed to read uploaded file: {err}"),
        }),
      )
    })?;
    bytes = Some(body.to_vec());
    break;
  }

  let bytes = bytes.ok_or_else(|| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "file field is required".to_string(),
      }),
    )
  })?;

  if let Ok(project_id) = repo_id.parse::<i64>() {
    let (project, namespace) = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "organization member permission is required",
    )
    .await?;

    let issue_id = if let Some(issue_id) = query.issue_id.as_deref() {
      let issue_id = parse_project_issue_id(issue_id)?;
      state
        .project_space
        .get_project_issue(project.id, issue_id)
        .await
        .map_err(map_project_space_error)?
        .ok_or_else(|| {
          (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
              message: "issue not found".to_string(),
            }),
          )
        })?;
      Some(issue_id.to_string())
    } else {
      None
    };

    let uploaded = state
      .services
      .repository
      .upload_project_issue_attachment(
        namespace.full_path.as_str(),
        project.path_key.as_str(),
        issue_id.as_deref(),
        file_name,
        content_type,
        bytes,
      )
      .await
      .map_err(map_repository_service_error)?;

    return Ok((
      StatusCode::CREATED,
      Json(issue_attachment_upload_view(uploaded)),
    ));
  }

  let uploaded = state
    .services
    .repository
    .upload_issue_attachment(UploadIssueAttachmentInput {
      repo_id,
      issue_id: query.issue_id,
      file_name,
      content_type,
      bytes,
      current_user_id: current_user.user_id,
    })
    .await
    .map_err(map_repository_service_error)?;

  Ok((
    StatusCode::CREATED,
    Json(issue_attachment_upload_view(uploaded)),
  ))
}

#[utoipa::path(
  get,
  path = "/{repo_id}/tree",
  tag = "Repositories",
  params(ListRepositoryTreeQuery),
  responses(
    (status = 200, description = "List repository tree entries", body = ApiResponse<Vec<RepositoryTreeEntryView>>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository or path not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Repositories",
  params(RepositoryBlobQuery),
  responses(
    (status = 200, description = "Read repository blob content", body = ApiResponse<RepositoryBlobView>),
    (status = 400, description = "Invalid request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository or file not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Repositories",
  params(RepositoryReadmeQuery),
  responses(
    (status = 200, description = "Read repository root README", body = ApiResponse<RepositoryBlobView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "README not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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

#[utoipa::path(
  get,
  path = "/{repo_id}/languages",
  tag = "Repositories",
  params(RepositoryLanguagesQuery),
  responses(
    (status = 200, description = "Read repository language statistics", body = ApiResponse<RepositoryLanguagesView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Repository not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn get_repository_languages(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(repo_id): Path<String>,
  Query(query): Query<RepositoryLanguagesQuery>,
) -> Result<(StatusCode, Json<RepositoryLanguagesView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let (project, namespace) = require_project_repository_access(
      &state,
      current_user.user_id.as_str(),
      project_id,
      None,
      "you are not a member of this organization",
    )
    .await?;
    let branch_name = query
      .branch_name
      .unwrap_or_else(|| project.default_branch.clone());

    if let Some(snapshot) = state
      .project_space
      .get_project_language_snapshot(project_id, branch_name.as_str())
      .await
      .map_err(map_project_space_error)?
    {
      return Ok((
        StatusCode::OK,
        Json(project_language_snapshot_api_view(
          repo_id.as_str(),
          snapshot,
        )),
      ));
    }

    if state.repository_language_jobs.is_some() {
      enqueue_repository_language_job(&state, repo_id.as_str(), Some(branch_name.as_str())).await;
      return Ok((
        StatusCode::OK,
        Json(RepositoryLanguagesView {
          repository_id: repo_id,
          branch_name,
          status: "pending".to_string(),
          revision: None,
          analyzed_at: None,
          total_bytes: 0,
          languages: Vec::new(),
        }),
      ));
    }

    let stats = state
      .services
      .git_backend
      .analyze_languages(
        namespace.full_path.as_str(),
        project.path_key.as_str(),
        branch_name.as_str(),
      )
      .await
      .map_err(map_repository_content_error)?;

    let total_bytes = stats.iter().map(|item| item.bytes).sum::<u64>();
    let languages = stats
      .into_iter()
      .map(|item| {
        let percentage = if total_bytes == 0 {
          0.0
        } else {
          let raw = (item.bytes as f64) * 100.0 / (total_bytes as f64);
          (raw * 100.0).round() / 100.0
        };
        RepositoryLanguageItemView {
          language: item.language,
          bytes: item.bytes,
          percentage,
        }
      })
      .collect::<Vec<_>>();

    return Ok((
      StatusCode::OK,
      Json(RepositoryLanguagesView {
        repository_id: repo_id,
        branch_name,
        status: "ready".to_string(),
        revision: None,
        analyzed_at: Some(chrono::Utc::now().to_rfc3339()),
        total_bytes,
        languages,
      }),
    ));
  }

  let repository = state
    .services
    .repository
    .require_repo_access(
      current_user.user_id.as_str(),
      repo_id.as_str(),
      RequiredOrganizationRole::Member,
    )
    .await
    .map_err(map_repository_service_error)?;
  let branch_name = query
    .branch_name
    .unwrap_or_else(|| repository.default_branch.clone());

  let snapshot = RepositoryLanguageSnapshotsRepository::new(state.db_conn.clone())
    .find_latest_snapshot_by_repo_and_branch(repo_id.as_str(), branch_name.as_str())
    .await
    .map_err(|err| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          message: format!("failed to load repository language snapshot: {err}"),
        }),
      )
    })?;

  let Some(snapshot) = snapshot else {
    enqueue_repository_language_job(&state, repo_id.as_str(), Some(branch_name.as_str())).await;
    return Ok((
      StatusCode::OK,
      Json(RepositoryLanguagesView {
        repository_id: repo_id,
        branch_name,
        status: "pending".to_string(),
        revision: None,
        analyzed_at: None,
        total_bytes: 0,
        languages: Vec::new(),
      }),
    ));
  };

  let items = RepositoryLanguageSnapshotsRepository::new(state.db_conn.clone())
    .list_snapshot_items_by_snapshot_id(snapshot.id.as_str())
    .await
    .map_err(|err| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          message: format!("failed to load repository language snapshot items: {err}"),
        }),
      )
    })?;

  let total_bytes = snapshot.total_bytes.max(0) as u64;
  let languages = items
    .into_iter()
    .map(|item| {
      let bytes = item.bytes.max(0) as u64;
      let percentage = if total_bytes == 0 {
        0.0
      } else {
        let raw = (bytes as f64) * 100.0 / (total_bytes as f64);
        (raw * 100.0).round() / 100.0
      };
      RepositoryLanguageItemView {
        language: item.language,
        bytes,
        percentage,
      }
    })
    .collect();

  Ok((
    StatusCode::OK,
    Json(RepositoryLanguagesView {
      repository_id: repo_id,
      branch_name,
      status: "ready".to_string(),
      revision: Some(snapshot.revision),
      analyzed_at: Some(snapshot.analyzed_at.to_rfc3339()),
      total_bytes,
      languages,
    }),
  ))
}

pub fn repo_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_repositories])
    .routes(routes![get_repository])
    .routes(routes![create_repository])
    .routes(routes![delete_repository])
    .routes(routes![list_branches])
    .routes(routes![create_branch])
    .routes(routes![protect_branch])
    .routes(routes![unprotect_branch])
    .routes(routes![list_commits])
    .routes(routes![create_commit])
    .routes(routes![create_file_commit])
    .routes(routes![list_issues])
    .routes(routes![create_issue])
    .routes(routes![get_issue_by_number])
    .routes(routes![update_issue])
    .routes(routes![list_issue_comments])
    .routes(routes![create_issue_comment])
    .routes(routes![upload_issue_attachment])
    .routes(routes![list_repository_tree])
    .routes(routes![read_repository_blob])
    .routes(routes![read_repository_readme])
    .routes(routes![get_repository_languages])
}

async fn list_project_repositories(
  state: AppState,
  current_user: CurrentUser,
  query: ListRepositoriesQuery,
) -> Result<(StatusCode, Json<Page<RepositoryView>>), (StatusCode, Json<ErrorResponse>)> {
  let ids = parse_csv_ids(query.ids.as_deref());
  let has_ids_filter = !ids.is_empty();
  let ids_filter = ids.into_iter().collect::<std::collections::HashSet<_>>();
  let pagination = if has_ids_filter {
    None
  } else {
    Some(resolve_pagination(query.page, query.page_size, 50, 200)?)
  };
  let namespace_filter = query
    .organization_id
    .and_then(|value| value.parse::<i64>().ok());
  let is_super_admin =
    current_user_is_repo_super_admin(&state, current_user.user_id.as_str()).await?;

  let mut data = Vec::new();
  for project in state
    .project_space
    .list_projects()
    .await
    .map_err(map_project_space_error)?
  {
    if let Some(namespace_id) = namespace_filter
      && project.namespace_id != namespace_id
    {
      continue;
    }

    let Some(namespace) = state
      .project_space
      .get_namespace(project.namespace_id)
      .await
      .map_err(map_project_space_error)?
    else {
      continue;
    };

    if !is_super_admin {
      let role = state
        .project_space
        .get_namespace_role(namespace.id, current_user.user_id.as_str())
        .await
        .map_err(map_project_space_error)?;
      let owns_namespace =
        namespace.owner_user_id.as_deref() == Some(current_user.user_id.as_str());
      if role.is_none() && !owns_namespace {
        continue;
      }
    }

    let view = project_repository_view(&project, &namespace, public_base_url(&state).as_str());
    if has_ids_filter && !ids_filter.contains(&view.id) {
      continue;
    }
    data.push(view);
  }

  if has_ids_filter {
    return Ok((StatusCode::OK, Json(to_single_page(data))));
  }

  let pagination = pagination.ok_or_else(|| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        message: "pagination state is invalid".to_string(),
      }),
    )
  })?;
  let total = data.len() as u64;
  let offset = ((pagination.page - 1) * pagination.page_size) as usize;
  let data = data
    .into_iter()
    .skip(offset)
    .take(pagination.page_size as usize)
    .collect::<Vec<_>>();
  Ok((StatusCode::OK, Json(to_page(data, total, pagination))))
}

async fn get_project_repository_view(
  state: &AppState,
  current_user: &CurrentUser,
  project_id: i64,
) -> Result<RepositoryView, (StatusCode, Json<ErrorResponse>)> {
  let project = state
    .project_space
    .get_project(project_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "repository not found".to_string(),
        }),
      )
    })?;
  let namespace = state
    .project_space
    .get_namespace(project.namespace_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "organization not found".to_string(),
        }),
      )
    })?;

  if !current_user_is_repo_super_admin(state, current_user.user_id.as_str()).await? {
    let role = state
      .project_space
      .get_namespace_role(namespace.id, current_user.user_id.as_str())
      .await
      .map_err(map_project_space_error)?;
    let owns_namespace = namespace.owner_user_id.as_deref() == Some(current_user.user_id.as_str());
    if role.is_none() && !owns_namespace {
      return Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
          message: "you are not a member of this organization".to_string(),
        }),
      ));
    }
  }

  Ok(project_repository_view(
    &project,
    &namespace,
    public_base_url(state).as_str(),
  ))
}

async fn create_project_repository(
  state: &AppState,
  current_user: &CurrentUser,
  namespace_id: i64,
  payload: CreateRepositoryRequest,
) -> Result<RepositoryView, (StatusCode, Json<ErrorResponse>)> {
  let namespace = state
    .project_space
    .get_namespace(namespace_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "organization not found".to_string(),
        }),
      )
    })?;

  let is_super_admin =
    current_user_is_repo_super_admin(state, current_user.user_id.as_str()).await?;
  if !is_super_admin {
    let role = state
      .project_space
      .get_namespace_role(namespace.id, current_user.user_id.as_str())
      .await
      .map_err(map_project_space_error)?;
    let owns_namespace = namespace.owner_user_id.as_deref() == Some(current_user.user_id.as_str());
    let allowed = role
      .as_deref()
      .is_some_and(|value| matches!(value, "developer" | "maintainer" | "owner"));
    if !allowed && !owns_namespace {
      return Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
          message: "organization developer permission is required".to_string(),
        }),
      ));
    }
  }

  let project = state
    .project_space
    .create_project(CreateProjectCommand {
      namespace_id,
      path_key: payload.key,
      name: payload.name,
      description: payload.description,
      visibility: payload.visibility.unwrap_or_else(|| "private".to_string()),
      default_branch: payload.default_branch,
      actor_user_id: current_user.user_id.clone(),
    })
    .await
    .map_err(map_project_space_error)?;

  let project_repo_id = project.id.to_string();
  enqueue_repository_language_job(
    state,
    project_repo_id.as_str(),
    Some(project.default_branch.as_str()),
  )
  .await;

  Ok(project_repository_view(
    &project,
    &namespace,
    public_base_url(state).as_str(),
  ))
}

async fn require_project_repository_access(
  state: &AppState,
  user_id: &str,
  project_id: i64,
  accepted_roles: Option<&[&str]>,
  denied_message: &str,
) -> Result<
  (
    application::project::ProjectView,
    application::namespace::NamespaceView,
  ),
  (StatusCode, Json<ErrorResponse>),
> {
  let project = state
    .project_space
    .get_project(project_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "repository not found".to_string(),
        }),
      )
    })?;
  let namespace = state
    .project_space
    .get_namespace(project.namespace_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "organization not found".to_string(),
        }),
      )
    })?;

  if current_user_is_repo_super_admin(state, user_id).await? {
    return Ok((project, namespace));
  }

  let role = state
    .project_space
    .get_namespace_role(namespace.id, user_id)
    .await
    .map_err(map_project_space_error)?;
  let owns_namespace = namespace.owner_user_id.as_deref() == Some(user_id);
  let allowed = match accepted_roles {
    Some(accepted_roles) => role
      .as_deref()
      .is_some_and(|value| accepted_roles.iter().any(|item| *item == value)),
    None => role.is_some(),
  };

  if !allowed && !owns_namespace {
    return Err((
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        message: denied_message.to_string(),
      }),
    ));
  }

  Ok((project, namespace))
}

fn parse_project_issue_id(issue_id: &str) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
  issue_id.parse::<i64>().map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "issue_id must be a numeric id for project-backed repositories".to_string(),
      }),
    )
  })
}

fn should_use_project_space_repo_api(
  requested_organization_id: Option<&str>,
  current_user: &CurrentUser,
  include_all: bool,
) -> bool {
  include_all
    || requested_organization_id.is_some_and(|value| value.parse::<i64>().is_ok())
    || current_user
      .organization_id
      .as_deref()
      .is_some_and(|value| value.parse::<i64>().is_ok())
}

async fn current_user_is_repo_super_admin(
  state: &AppState,
  user_id: &str,
) -> Result<bool, (StatusCode, Json<ErrorResponse>)> {
  let user = state
    .services
    .user
    .get_current_user(user_id)
    .await
    .map_err(map_user_service_error_as_repo_error)?;
  Ok(state.services.user.is_super_admin(&user))
}

fn map_user_service_error_as_repo_error(
  err: crate::service::user_service::UserServiceError,
) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    crate::service::user_service::UserServiceError::BadRequest(message) => {
      (StatusCode::BAD_REQUEST, message)
    }
    crate::service::user_service::UserServiceError::Unauthorized(message) => {
      (StatusCode::UNAUTHORIZED, message)
    }
    crate::service::user_service::UserServiceError::Forbidden(message) => {
      (StatusCode::FORBIDDEN, message)
    }
    crate::service::user_service::UserServiceError::NotFound(message) => {
      (StatusCode::NOT_FOUND, message)
    }
    crate::service::user_service::UserServiceError::Conflict(message) => {
      (StatusCode::CONFLICT, message)
    }
    crate::service::user_service::UserServiceError::Internal(message) => {
      (StatusCode::INTERNAL_SERVER_ERROR, message)
    }
  };
  (status, Json(ErrorResponse { message }))
}

fn project_repository_view(
  project: &application::project::ProjectView,
  namespace: &application::namespace::NamespaceView,
  base_url: &str,
) -> RepositoryView {
  RepositoryView {
    id: project.id.to_string(),
    uuid: project.id.to_string(),
    organization_id: namespace.id.to_string(),
    key: project.path_key.clone(),
    name: project.name.clone(),
    description: project.description.clone(),
    visibility: project.visibility.clone(),
    default_branch: project.default_branch.clone(),
    clone_http_url: format!("{base_url}/{}.git", project.full_path),
  }
}
async fn set_branch_protection(
  state: &AppState,
  current_user: &CurrentUser,
  repo_id: String,
  branch_name: String,
  is_protected: bool,
) -> Result<(StatusCode, Json<BranchView>), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let _ = require_project_repository_access(
      state,
      current_user.user_id.as_str(),
      project_id,
      Some(&["owner"]),
      "organization owner permission is required",
    )
    .await?;
    let branch = state
      .project_space
      .set_project_branch_protection(SetProjectBranchProtectionCommand {
        project_id,
        branch_name,
        is_protected,
      })
      .await
      .map_err(map_project_space_error)?;
    return Ok((
      StatusCode::OK,
      Json(project_branch_api_view(repo_id.as_str(), branch)),
    ));
  }

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
    uuid: model.uuid,
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

fn issue_view(model: repository_issues::Model) -> IssueView {
  IssueView {
    id: model.id,
    repository_id: model.repository_id,
    number: model.number,
    title: model.title,
    description: model.description,
    status: match model.status {
      repository_issues::RepositoryIssueStatus::Open => "open".to_string(),
      repository_issues::RepositoryIssueStatus::Closed => "closed".to_string(),
    },
    author_user_id: model.author_user_id,
    assignee_user_id: model.assignee_user_id,
    created_at: model.created_at.to_rfc3339(),
    updated_at: model.updated_at.to_rfc3339(),
    closed_at: model.closed_at.map(|time| time.to_rfc3339()),
  }
}

fn issue_comment_view(model: repository_issue_comments::Model) -> IssueCommentView {
  IssueCommentView {
    id: model.id,
    issue_id: model.issue_id,
    author_user_id: model.author_user_id,
    content: model.content,
    created_at: model.created_at.to_rfc3339(),
    updated_at: model.updated_at.to_rfc3339(),
  }
}

fn project_branch_api_view(repo_id: &str, model: ProjectSpaceBranchView) -> BranchView {
  BranchView {
    repository_id: repo_id.to_string(),
    name: model.name,
    is_protected: model.is_protected,
    last_commit_sha: model.last_commit_sha,
  }
}

fn project_issue_api_view(model: ProjectSpaceIssueView) -> IssueView {
  IssueView {
    id: model.id.to_string(),
    repository_id: model.project_id.to_string(),
    number: i32::try_from(model.iid).unwrap_or(i32::MAX),
    title: model.title,
    description: model.description,
    status: model.state,
    author_user_id: model.author_user_id,
    assignee_user_id: model.assignee_user_id,
    created_at: unix_seconds_to_rfc3339(model.created_at_unix),
    updated_at: unix_seconds_to_rfc3339(model.updated_at_unix),
    closed_at: model.closed_at_unix.map(unix_seconds_to_rfc3339),
  }
}

fn project_issue_comment_api_view(model: ProjectSpaceIssueCommentView) -> IssueCommentView {
  IssueCommentView {
    id: model.id.to_string(),
    issue_id: model.project_issue_id.to_string(),
    author_user_id: model.author_user_id,
    content: model.body,
    created_at: unix_seconds_to_rfc3339(model.created_at_unix),
    updated_at: unix_seconds_to_rfc3339(model.updated_at_unix),
  }
}

fn project_language_snapshot_api_view(
  repo_id: &str,
  snapshot: ProjectSpaceLanguageSnapshotView,
) -> RepositoryLanguagesView {
  let total_bytes = snapshot.total_bytes;
  let languages = snapshot
    .items
    .into_iter()
    .map(|item| {
      let percentage = if total_bytes == 0 {
        0.0
      } else {
        let raw = (item.bytes as f64) * 100.0 / (total_bytes as f64);
        (raw * 100.0).round() / 100.0
      };
      RepositoryLanguageItemView {
        language: item.language,
        bytes: item.bytes,
        percentage,
      }
    })
    .collect::<Vec<_>>();

  RepositoryLanguagesView {
    repository_id: repo_id.to_string(),
    branch_name: snapshot.branch_name,
    status: "ready".to_string(),
    revision: Some(snapshot.revision),
    analyzed_at: Some(unix_seconds_to_rfc3339(snapshot.analyzed_at_unix)),
    total_bytes,
    languages,
  }
}

fn issue_attachment_upload_view(output: UploadIssueAttachmentOutput) -> IssueAttachmentUploadView {
  IssueAttachmentUploadView {
    url: output.url,
    object_key: output.object_key,
    file_name: output.file_name,
    content_type: output.content_type,
    size: output.size,
    markdown: output.markdown,
  }
}

fn map_project_space_error(err: ProjectSpaceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    ProjectSpaceError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
    ProjectSpaceError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    ProjectSpaceError::Conflict(message) => (StatusCode::CONFLICT, message),
    ProjectSpaceError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
    ProjectSpaceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  };
  (status, Json(ErrorResponse { message }))
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

async fn enqueue_repository_language_job(
  state: &AppState,
  repository_id: &str,
  branch_name: Option<&str>,
) {
  let Some(job_client) = state.repository_language_jobs.as_ref() else {
    return;
  };

  let enqueue_result = if let Ok(project_id) = repository_id.parse::<i64>() {
    job_client
      .enqueue_project_branch(project_id, branch_name)
      .await
  } else {
    job_client
      .enqueue_repository_branch(repository_id, branch_name)
      .await
  };

  if let Err(err) = enqueue_result {
    warn!(
      repository_id = repository_id,
      branch = branch_name.unwrap_or(""),
      error = err,
      "failed to enqueue repository language job"
    );
  }
}

async fn resolve_repo_storage_context(
  state: &AppState,
  user_id: &str,
  repo_id: &str,
  branch_name: Option<String>,
) -> Result<(String, String, String), (StatusCode, Json<ErrorResponse>)> {
  if let Ok(project_id) = repo_id.parse::<i64>() {
    let project = state
      .project_space
      .get_project(project_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "repository not found".to_string(),
          }),
        )
      })?;
    let namespace = state
      .project_space
      .get_namespace(project.namespace_id)
      .await
      .map_err(map_project_space_error)?
      .ok_or_else(|| {
        (
          StatusCode::NOT_FOUND,
          Json(ErrorResponse {
            message: "organization not found".to_string(),
          }),
        )
      })?;

    if !current_user_is_repo_super_admin(state, user_id).await? {
      let role = state
        .project_space
        .get_namespace_role(namespace.id, user_id)
        .await
        .map_err(map_project_space_error)?;
      let owns_namespace = namespace.owner_user_id.as_deref() == Some(user_id);
      if role.is_none() && !owns_namespace {
        return Err((
          StatusCode::FORBIDDEN,
          Json(ErrorResponse {
            message: "you are not a member of this organization".to_string(),
          }),
        ));
      }
    }

    let branch = branch_name.unwrap_or(project.default_branch);
    return Ok((namespace.full_path, project.path_key, branch));
  }

  let repository = state
    .services
    .repository
    .require_repo_access(user_id, repo_id, RequiredOrganizationRole::Member)
    .await
    .map_err(map_repository_service_error)?;

  let organization = OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_id(repository.organization_id.as_str())
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

  state
    .services
    .repository
    .ensure_repository_storage_ready(organization.key.as_str(), &repository)
    .await
    .map_err(map_repository_service_error)?;

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
    crate::service::git_backend_service::GitBackendError::InvalidRepositoryPath => (
      StatusCode::BAD_REQUEST,
      "invalid repository path".to_string(),
    ),
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
