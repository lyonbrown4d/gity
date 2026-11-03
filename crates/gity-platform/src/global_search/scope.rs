use super::SearchRequest;
use crate::{AppState, auth};
use axum::http::HeaderMap;
use gity_core::{AppError, Principal};
use gity_projects::SearchProject;
use sha2::{Digest, Sha256};

pub struct Scope {
  pub actor: Principal,
  pub projects: Vec<SearchProject>,
  pub confidential_projects: Vec<i64>,
  pub confidential_ids: Vec<i64>,
  pub fingerprint: String,
}
fn id(value: &str) -> Result<i64, AppError> {
  value
    .parse::<i64>()
    .ok()
    .filter(|v| *v > 0)
    .ok_or_else(|| AppError::bad_request("invalid search identifier"))
}
pub async fn resolve(
  s: &AppState,
  h: &HeaderMap,
  input: &SearchRequest,
) -> Result<Scope, AppError> {
  let actor = auth::principal(s, h).await?;
  if let Some(token) = auth::bearer(h).filter(|token| token.starts_with("gity_oauth_")) {
    let audience = format!("{}/mcp", s.settings.base_url.trim_end_matches('/'));
    let identity = s
      .oauth
      .validate_bearer(token, &audience)
      .await
      .map_err(|_| AppError::unauthorized("invalid or expired credentials"))?
      .ok_or_else(|| AppError::unauthorized("invalid or expired credentials"))?;
    if !identity
      .scopes
      .iter()
      .any(|v| matches!(v.as_str(), "read_api" | "write_api"))
    {
      return Err(AppError::forbidden(
        "credential scope does not permit global search",
      ));
    }
  }
  let projects = input
    .project_ids
    .iter()
    .map(|v| id(v))
    .collect::<Result<Vec<_>, _>>()?;
  let org = input.organization_id.as_deref().map(id).transpose()?;
  let projects = s.projects.search_scope(&actor, org, &projects).await?;
  let restricted = projects
    .iter()
    .filter(|v| v.level < 20)
    .map(|v| v.project.id)
    .collect::<Vec<_>>();
  let confidential_projects = projects
    .iter()
    .filter(|v| v.level >= 20)
    .map(|v| v.project.id)
    .collect::<Vec<_>>();
  let mut confidential_ids = s
    .collaboration
    .search_confidential_ids(actor.user_id, &restricted)
    .await?;
  confidential_ids.sort_unstable();
  let mut hash = Sha256::new();
  hash.update(actor.user_id.to_be_bytes());
  for p in &projects {
    hash.update(p.project.id.to_be_bytes());
    hash.update(p.level.to_be_bytes());
    hash.update(p.project.visibility.as_bytes());
    hash.update(p.project.default_branch.as_bytes());
  }
  for id in &confidential_ids {
    hash.update(id.to_be_bytes());
  }
  Ok(Scope {
    actor,
    projects,
    confidential_projects,
    confidential_ids,
    fingerprint: hex::encode(hash.finalize()),
  })
}
