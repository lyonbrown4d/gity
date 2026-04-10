use crate::http::app_state::AppState;
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::{RequiredOrganizationRole, require_organization_role};
use crate::service::git_backend_service::GitBackendError;
use crate::service::project_space_service::ProjectSpaceError;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use entity::{organizations, repositories};
use git::http;
use models::constants::member_role;
use repository::{OrganizationsRepository, RepositoriesRepository};
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
pub struct InfoRefsParams {
  pub service: String,
}

/// GET /git/:owner/:repo/info/refs?service=git-<name>
pub async fn info_refs(
  Path(namespace_and_repo): Path<String>,
  Query(params): Query<InfoRefsParams>,
  State(app_state): State<AppState>,
  headers: HeaderMap,
) -> impl IntoResponse {
  let (owner, repo) = match parse_namespace_and_repo(namespace_and_repo.as_str()) {
    Some(parts) => parts,
    None => return (StatusCode::BAD_REQUEST, "invalid repository path").into_response(),
  };
  let service = match parse_git_service(params.service.as_str()) {
    Some(service) => service,
    None => {
      return (
        StatusCode::BAD_REQUEST,
        "service must be git-upload-pack or git-receive-pack",
      )
        .into_response();
    }
  };

  if owner.contains('/') {
    if service == "receive-pack" {
      let current_user = match resolve_current_user_from_headers(&app_state, &headers).await {
        Ok(user) => user,
        Err(err) => return err.into_response(),
      };
      if let Err(err) = require_namespace_project_write_access(
        &app_state,
        &current_user,
        owner.as_str(),
        repo.as_str(),
      )
      .await
      {
        return err.into_response();
      }
    } else if !allow_anonymous_read(&app_state) {
      let current_user = match resolve_current_user_from_headers(&app_state, &headers).await {
        Ok(user) => user,
        Err(err) => return err.into_response(),
      };
      if let Err(err) = require_namespace_project_read_access(
        &app_state,
        &current_user,
        owner.as_str(),
        repo.as_str(),
      )
      .await
      {
        return err.into_response();
      }
    }
  } else if service == "receive-pack" {
    let current_user = match resolve_current_user_from_headers(&app_state, &headers).await {
      Ok(user) => user,
      Err(err) => return err.into_response(),
    };
    if let Err(err) = require_repository_permission(
      &app_state,
      &current_user,
      owner.as_str(),
      repo.as_str(),
      RequiredOrganizationRole::Owner,
    )
    .await
    {
      return err.into_response();
    }
  } else if !allow_anonymous_read(&app_state) {
    let current_user = match resolve_current_user_from_headers(&app_state, &headers).await {
      Ok(user) => user,
      Err(err) => return err.into_response(),
    };
    if let Err(err) = require_repository_permission(
      &app_state,
      &current_user,
      owner.as_str(),
      repo.as_str(),
      RequiredOrganizationRole::Member,
    )
    .await
    {
      return err.into_response();
    }
  }

  let repo_path = match app_state
    .services
    .git_backend
    .resolve_repo_path(owner.as_str(), repo.as_str())
  {
    Ok(path) => path,
    Err(err) => {
      let (status, message) = map_git_backend_error(err);
      return (status, message).into_response();
    }
  };

  match http::advertise_refs(&repo_path, service) {
    Ok(body) => {
      let mut headers = HeaderMap::new();
      headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));

      let content_type = format!("application/x-git-{}-advertisement", service);
      match HeaderValue::from_str(content_type.as_str()) {
        Ok(content_type_value) => {
          headers.insert(header::CONTENT_TYPE, content_type_value);
          (StatusCode::OK, headers, body).into_response()
        }
        Err(_) => (
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to build response content type",
        )
          .into_response(),
      }
    }
    Err(err) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("failed to advertise refs: {err}"),
    )
      .into_response(),
  }
}

/// POST /git/:owner/:repo/git-upload-pack
pub async fn upload_pack(
  Path(namespace_and_repo): Path<String>,
  State(app_state): State<AppState>,
  headers: HeaderMap,
  body: Bytes,
) -> impl IntoResponse {
  let (owner, repo) = match parse_namespace_and_repo(namespace_and_repo.as_str()) {
    Some(parts) => parts,
    None => return (StatusCode::BAD_REQUEST, "invalid repository path").into_response(),
  };

  if owner.contains('/') {
    if !allow_anonymous_read(&app_state) {
      let current_user = match resolve_current_user_from_headers(&app_state, &headers).await {
        Ok(user) => user,
        Err(err) => return err.into_response(),
      };
      if let Err(err) = require_namespace_project_read_access(
        &app_state,
        &current_user,
        owner.as_str(),
        repo.as_str(),
      )
      .await
      {
        return err.into_response();
      }
    }
  } else if !allow_anonymous_read(&app_state) {
    let current_user = match resolve_current_user_from_headers(&app_state, &headers).await {
      Ok(user) => user,
      Err(err) => return err.into_response(),
    };
    if let Err(err) = require_repository_permission(
      &app_state,
      &current_user,
      owner.as_str(),
      repo.as_str(),
      RequiredOrganizationRole::Member,
    )
    .await
    {
      return err.into_response();
    }
  }

  service_pack_inner(
    owner.as_str(),
    repo.as_str(),
    "upload-pack",
    &app_state,
    body,
  )
  .await
}

/// POST /git/:owner/:repo/git-receive-pack
pub async fn receive_pack(
  Path(namespace_and_repo): Path<String>,
  State(app_state): State<AppState>,
  current_user: CurrentUser,
  body: Bytes,
) -> impl IntoResponse {
  let (owner, repo) = match parse_namespace_and_repo(namespace_and_repo.as_str()) {
    Some(parts) => parts,
    None => return (StatusCode::BAD_REQUEST, "invalid repository path").into_response(),
  };

  if owner.contains('/') {
    if let Err(err) = require_namespace_project_write_access(
      &app_state,
      &current_user,
      owner.as_str(),
      repo.as_str(),
    )
    .await
    {
      return err.into_response();
    }
    if let Err(err) = enforce_project_receive_pack_branch_protection(
      &app_state,
      &current_user,
      owner.as_str(),
      repo.as_str(),
      body.as_ref(),
    )
    .await
    {
      return err.into_response();
    }

    let repo_path = match app_state
      .services
      .git_backend
      .resolve_repo_path(owner.as_str(), repo.as_str())
    {
      Ok(path) => path,
      Err(err) => {
        let (status, message) = map_git_backend_error(err);
        return (status, message).into_response();
      }
    };

    let output = match app_state
      .services
      .git_backend
      .run_stateless_rpc(repo_path.as_path(), "receive-pack", &body)
      .await
    {
      Ok(output) => output,
      Err(err) => {
        let (status, message) = map_git_backend_error(err);
        return (status, message).into_response();
      }
    };

    let updated_branches = parse_receive_pack_branch_updates(body.as_ref()).unwrap_or_default();
    if let Err(err) =
      sync_project_receive_pack_metadata(&app_state, owner.as_str(), repo.as_str()).await
    {
      warn!(
        project_owner = owner,
        project_repo = repo,
        error = err.1,
        "receive-pack succeeded but failed to sync project branch metadata"
      );
    } else if let Err(err) = enqueue_project_receive_pack_language_jobs(
      &app_state,
      owner.as_str(),
      repo.as_str(),
      &updated_branches,
    )
    .await
    {
      warn!(
        project_owner = owner,
        project_repo = repo,
        error = err.1,
        "receive-pack succeeded but failed to enqueue project language jobs"
      );
    }

    return build_service_response("receive-pack", output);
  }

  let (organization, repository) =
    match require_receive_pack_permission(&app_state, &current_user, owner.as_str(), repo.as_str())
      .await
    {
      Ok(data) => data,
      Err((status, message)) => return (status, message).into_response(),
    };

  let repo_path = match app_state
    .services
    .git_backend
    .resolve_repo_path(owner.as_str(), repo.as_str())
  {
    Ok(path) => path,
    Err(err) => {
      let (status, message) = map_git_backend_error(err);
      return (status, message).into_response();
    }
  };

  let output = match app_state
    .services
    .git_backend
    .run_stateless_rpc(repo_path.as_path(), "receive-pack", &body)
    .await
  {
    Ok(output) => output,
    Err(err) => {
      let (status, message) = map_git_backend_error(err);
      return (status, message).into_response();
    }
  };

  if let Err(err) = app_state
    .services
    .git_backend
    .sync_branches_from_refs(repository.id.as_str(), repo_path.as_path())
    .await
  {
    warn!(
      organization = organization.key,
      repository = repository.key,
      error = err.to_string(),
      "receive-pack succeeded but failed to sync branch metadata"
    );
  }

  if let Some(job_client) = app_state.repository_language_jobs.as_ref()
    && let Err(err) = job_client
      .enqueue_repository_branch(
        repository.id.as_str(),
        Some(repository.default_branch.as_str()),
      )
      .await
  {
    warn!(
      organization = organization.key,
      repository = repository.key,
      error = err,
      "receive-pack succeeded but failed to enqueue repository language job"
    );
  }

  build_service_response("receive-pack", output)
}

fn parse_git_service(input: &str) -> Option<&str> {
  let normalized = input.strip_prefix("git-").unwrap_or(input);
  match normalized {
    "upload-pack" | "receive-pack" => Some(normalized),
    _ => None,
  }
}

fn parse_namespace_and_repo(value: &str) -> Option<(String, String)> {
  let trimmed = value.trim_matches('/');
  if trimmed.is_empty() {
    return None;
  }

  let mut parts = trimmed.rsplitn(2, '/');
  let repo = parts.next()?.trim();
  let owner = parts.next()?.trim();
  if owner.is_empty() || repo.is_empty() {
    return None;
  }

  Some((owner.to_string(), repo.to_string()))
}

async fn service_pack_inner(
  owner: &str,
  repo: &str,
  service: &str,
  app_state: &AppState,
  body: Bytes,
) -> Response {
  let repo_path = match app_state
    .services
    .git_backend
    .resolve_repo_path(owner, repo)
  {
    Ok(path) => path,
    Err(err) => {
      let (status, message) = map_git_backend_error(err);
      return (status, message).into_response();
    }
  };

  let output = match app_state
    .services
    .git_backend
    .run_stateless_rpc(repo_path.as_path(), service, &body)
    .await
  {
    Ok(output) => output,
    Err(err) => {
      let (status, message) = map_git_backend_error(err);
      return (status, message).into_response();
    }
  };

  build_service_response(service, output)
}

fn build_service_response(service: &str, output: Vec<u8>) -> Response {
  let mut headers = HeaderMap::new();
  headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
  let content_type = format!("application/x-git-{service}-result");
  match HeaderValue::from_str(content_type.as_str()) {
    Ok(content_type_value) => {
      headers.insert(header::CONTENT_TYPE, content_type_value);
      (StatusCode::OK, headers, output).into_response()
    }
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      "failed to build response content type",
    )
      .into_response(),
  }
}

async fn require_receive_pack_permission(
  state: &AppState,
  current_user: &CurrentUser,
  owner: &str,
  repo: &str,
) -> Result<(organizations::Model, repositories::Model), (StatusCode, String)> {
  require_repository_permission(
    state,
    current_user,
    owner,
    repo,
    RequiredOrganizationRole::Owner,
  )
  .await
}

async fn require_repository_permission(
  state: &AppState,
  current_user: &CurrentUser,
  owner: &str,
  repo: &str,
  required_role: RequiredOrganizationRole,
) -> Result<(organizations::Model, repositories::Model), (StatusCode, String)> {
  let organization = OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_key(owner)
    .await
    .map_err(|err| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to load organization: {err}"),
      )
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "organization not found".to_string()))?;

  let repo_key = repo.strip_suffix(".git").unwrap_or(repo);
  let repository = RepositoriesRepository::new(state.db_conn.clone())
    .find_active_repository_by_org_and_key(organization.id.as_str(), repo_key)
    .await
    .map_err(|err| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to load repository metadata: {err}"),
      )
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "repository not found".to_string()))?;

  require_organization_role(
    &state.db_conn,
    current_user.user_id.as_str(),
    organization.id.as_str(),
    required_role,
  )
  .await
  .map_err(|err| (err.status, err.message))?;

  Ok((organization, repository))
}

async fn require_namespace_project_read_access(
  state: &AppState,
  current_user: &CurrentUser,
  owner: &str,
  repo: &str,
) -> Result<(), (StatusCode, String)> {
  require_namespace_project_access(
    state,
    current_user,
    owner,
    repo,
    &[
      member_role::GUEST,
      member_role::REPORTER,
      member_role::DEVELOPER,
      member_role::MAINTAINER,
      member_role::OWNER,
    ],
    "you are not a member of this project namespace",
  )
  .await
}

async fn require_namespace_project_write_access(
  state: &AppState,
  current_user: &CurrentUser,
  owner: &str,
  repo: &str,
) -> Result<(), (StatusCode, String)> {
  require_namespace_project_access(
    state,
    current_user,
    owner,
    repo,
    &[
      member_role::DEVELOPER,
      member_role::MAINTAINER,
      member_role::OWNER,
    ],
    "project developer permission is required",
  )
  .await
}

async fn require_namespace_project_access(
  state: &AppState,
  current_user: &CurrentUser,
  owner: &str,
  repo: &str,
  accepted_roles: &[&str],
  forbidden_message: &str,
) -> Result<(), (StatusCode, String)> {
  if current_user_is_super_admin(state, current_user.user_id.as_str()).await? {
    return Ok(());
  }

  let repo_key = repo.strip_suffix(".git").unwrap_or(repo);
  let project_full_path = format!("{owner}/{repo_key}");
  let project = state
    .project_space
    .get_project_by_full_path(project_full_path.as_str())
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "project not found".to_string()))?;

  let role = state
    .project_space
    .get_namespace_role(project.namespace_id, current_user.user_id.as_str())
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| (StatusCode::FORBIDDEN, forbidden_message.to_string()))?;

  if accepted_roles.iter().any(|item| *item == role) {
    Ok(())
  } else {
    Err((StatusCode::FORBIDDEN, forbidden_message.to_string()))
  }
}

async fn enforce_project_receive_pack_branch_protection(
  state: &AppState,
  current_user: &CurrentUser,
  owner: &str,
  repo: &str,
  body: &[u8],
) -> Result<(), (StatusCode, String)> {
  if current_user_is_super_admin(state, current_user.user_id.as_str()).await? {
    return Ok(());
  }

  let repo_key = repo.strip_suffix(".git").unwrap_or(repo);
  let project_full_path = format!("{owner}/{repo_key}");
  let project = state
    .project_space
    .get_project_by_full_path(project_full_path.as_str())
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "project not found".to_string()))?;
  let namespace = state
    .project_space
    .get_namespace(project.namespace_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "namespace not found".to_string()))?;

  let is_owner = namespace.owner_user_id.as_deref() == Some(current_user.user_id.as_str())
    || state
      .project_space
      .get_namespace_role(namespace.id, current_user.user_id.as_str())
      .await
      .map_err(map_project_space_error)?
      .as_deref()
      == Some(member_role::OWNER);
  if is_owner {
    return Ok(());
  }

  for branch_name in parse_receive_pack_branch_updates(body)? {
    let branch = state
      .project_space
      .get_project_branch(project.id, branch_name.as_str())
      .await
      .map_err(map_project_space_error)?;
    if branch.is_some_and(|item| item.is_protected) {
      return Err((
        StatusCode::FORBIDDEN,
        "only organization owner can push to protected branch".to_string(),
      ));
    }
  }

  Ok(())
}

async fn sync_project_receive_pack_metadata(
  state: &AppState,
  owner: &str,
  repo: &str,
) -> Result<(), (StatusCode, String)> {
  let repo_key = repo.strip_suffix(".git").unwrap_or(repo);
  let project_full_path = format!("{owner}/{repo_key}");
  if let Some(project) = state
    .project_space
    .get_project_by_full_path(project_full_path.as_str())
    .await
    .map_err(map_project_space_error)?
  {
    let _ = state
      .project_space
      .list_project_branches(project.id)
      .await
      .map_err(map_project_space_error)?;
  }
  Ok(())
}

async fn enqueue_project_receive_pack_language_jobs(
  state: &AppState,
  owner: &str,
  repo: &str,
  updated_branches: &[String],
) -> Result<(), (StatusCode, String)> {
  let Some(job_client) = state.repository_language_jobs.as_ref() else {
    return Ok(());
  };

  let repo_key = repo.strip_suffix(".git").unwrap_or(repo);
  let project_full_path = format!("{owner}/{repo_key}");
  let Some(project) = state
    .project_space
    .get_project_by_full_path(project_full_path.as_str())
    .await
    .map_err(map_project_space_error)?
  else {
    return Ok(());
  };

  let branch_names = if updated_branches.is_empty() {
    vec![project.default_branch.clone()]
  } else {
    updated_branches.to_vec()
  };

  for branch_name in branch_names {
    if let Err(err) = job_client
      .enqueue_project_branch(project.id, Some(branch_name.as_str()))
      .await
    {
      return Err((StatusCode::INTERNAL_SERVER_ERROR, err));
    }
  }

  Ok(())
}

fn parse_receive_pack_branch_updates(body: &[u8]) -> Result<Vec<String>, (StatusCode, String)> {
  let mut offset = 0usize;
  let mut branches = std::collections::BTreeSet::new();

  while offset + 4 <= body.len() {
    let length = parse_pkt_line_length(&body[offset..offset + 4])?;
    offset += 4;
    if length == 0 {
      break;
    }
    if length < 4 {
      return Err((
        StatusCode::BAD_REQUEST,
        "invalid git receive-pack payload length".to_string(),
      ));
    }

    let payload_len = length - 4;
    if offset + payload_len > body.len() {
      return Err((
        StatusCode::BAD_REQUEST,
        "truncated git receive-pack payload".to_string(),
      ));
    }

    let payload = &body[offset..offset + payload_len];
    offset += payload_len;
    if payload.is_empty() {
      continue;
    }

    let payload = if let Some(position) = payload.iter().position(|item| *item == 0) {
      &payload[..position]
    } else {
      payload
    };
    let line = std::str::from_utf8(payload).map_err(|_| {
      (
        StatusCode::BAD_REQUEST,
        "git receive-pack payload is not valid utf-8".to_string(),
      )
    })?;
    let mut parts = line.split_whitespace();
    let _old_oid = parts.next();
    let _new_oid = parts.next();
    let Some(ref_name) = parts.next() else {
      continue;
    };
    if let Some(branch_name) = ref_name.strip_prefix("refs/heads/") {
      branches.insert(branch_name.to_string());
    }
  }

  Ok(branches.into_iter().collect())
}

fn parse_pkt_line_length(header: &[u8]) -> Result<usize, (StatusCode, String)> {
  let value = std::str::from_utf8(header).map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      "invalid git pkt-line header".to_string(),
    )
  })?;
  usize::from_str_radix(value, 16).map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      "invalid git pkt-line length".to_string(),
    )
  })
}

async fn current_user_is_super_admin(
  state: &AppState,
  user_id: &str,
) -> Result<bool, (StatusCode, String)> {
  let user = state
    .services
    .user
    .get_current_user(user_id)
    .await
    .map_err(map_user_service_error)?;
  Ok(state.services.user.is_super_admin(&user))
}

fn map_user_service_error(
  err: crate::service::user_service::UserServiceError,
) -> (StatusCode, String) {
  match err {
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
  }
}

fn map_project_space_error(err: ProjectSpaceError) -> (StatusCode, String) {
  match err {
    ProjectSpaceError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
    ProjectSpaceError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    ProjectSpaceError::Conflict(message) => (StatusCode::CONFLICT, message),
    ProjectSpaceError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
    ProjectSpaceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  }
}

async fn resolve_current_user_from_headers(
  state: &AppState,
  headers: &HeaderMap,
) -> Result<CurrentUser, (StatusCode, String)> {
  let token = headers
    .get(header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.strip_prefix("Bearer "))
    .ok_or_else(|| {
      (
        StatusCode::UNAUTHORIZED,
        "missing or invalid authorization header".to_string(),
      )
    })?;

  let claims = state
    .services
    .auth
    .verify_access_token_for_request(token)
    .await
    .map_err(|_| {
      (
        StatusCode::UNAUTHORIZED,
        "invalid, expired, or revoked token".to_string(),
      )
    })?;

  Ok(CurrentUser {
    user_id: claims.sub,
    organization_id: claims.org,
  })
}

fn allow_anonymous_read(state: &AppState) -> bool {
  state
    .config
    .git
    .as_ref()
    .and_then(|git| git.allow_anonymous_read)
    .unwrap_or(false)
}

fn map_git_backend_error(err: GitBackendError) -> (StatusCode, String) {
  match err {
    GitBackendError::InvalidRepositoryPath => (StatusCode::BAD_REQUEST, err.to_string()),
    GitBackendError::RepositoryNotFound => (StatusCode::NOT_FOUND, err.to_string()),
    GitBackendError::AlreadyExists(_) => (StatusCode::CONFLICT, err.to_string()),
    GitBackendError::StorageNotConfigured
    | GitBackendError::InvalidComponent(_)
    | GitBackendError::Io(_)
    | GitBackendError::Git(_)
    | GitBackendError::Db(_)
    | GitBackendError::Utf8(_) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
  }
}

pub fn git_routes() -> Router<AppState> {
  Router::new()
    .route("/{*namespace_and_repo}/info/refs", get(info_refs))
    .route("/{*namespace_and_repo}/git-upload-pack", post(upload_pack))
    .route(
      "/{*namespace_and_repo}/git-receive-pack",
      post(receive_pack),
    )
}
