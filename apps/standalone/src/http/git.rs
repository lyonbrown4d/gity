use crate::http::app_state::AppState;
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::{RequiredOrganizationRole, require_organization_role};
use crate::service::git_backend_service::GitBackendError;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use entity::{organizations, repositories};
use git::http;
use repository::{OrganizationsRepository, RepositoriesRepository};
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
pub struct InfoRefsParams {
  pub service: String,
}

/// GET /git/:owner/:repo/info/refs?service=git-<name>
pub async fn info_refs(
  Path((owner, repo)): Path<(String, String)>,
  Query(params): Query<InfoRefsParams>,
  State(app_state): State<AppState>,
  headers: HeaderMap,
) -> impl IntoResponse {
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

  if service == "receive-pack" {
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
  Path((owner, repo)): Path<(String, String)>,
  State(app_state): State<AppState>,
  headers: HeaderMap,
  body: Bytes,
) -> impl IntoResponse {
  if !allow_anonymous_read(&app_state) {
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
  Path((owner, repo)): Path<(String, String)>,
  State(app_state): State<AppState>,
  current_user: CurrentUser,
  body: Bytes,
) -> impl IntoResponse {
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
  let organization =
    OrganizationsRepository::find_active_organization_by_key(&state.db_conn, owner)
      .await
      .map_err(|err| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          format!("failed to load organization: {err}"),
        )
      })?
      .ok_or_else(|| (StatusCode::NOT_FOUND, "organization not found".to_string()))?;

  let repo_key = repo.strip_suffix(".git").unwrap_or(repo);
  let repository = RepositoriesRepository::find_active_repository_by_org_and_key(
    &state.db_conn,
    organization.id.as_str(),
    repo_key,
  )
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
    .route("/{owner}/{repo}/info/refs", get(info_refs))
    .route("/{owner}/{repo}/git-upload-pack", post(upload_pack))
    .route("/{owner}/{repo}/git-receive-pack", post(receive_pack))
}
