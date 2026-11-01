mod actions;
mod project_action;
use crate::{AppState, auth};
pub use actions::ProjectAction;
use axum::http::HeaderMap;
use gity_core::{AppError, Principal};
use gity_identity::MembershipRole;
pub use project_action::{ProjectActor, ProjectAuthorization, project_action};
use serde_json::Value;

pub async fn actor(state: &AppState, headers: &HeaderMap) -> Result<Principal, AppError> {
  auth::principal(state, headers).await
}
pub fn role_level(role: &str) -> i64 {
  MembershipRole::parse(role).level()
}
pub async fn level(state: &AppState, actor: &Principal, project: &Value) -> Result<i64, AppError> {
  if actor.is_super_admin {
    return Ok(100);
  }
  let id = project["id"]
    .as_i64()
    .ok_or_else(|| AppError::internal("invalid project"))?;
  let repository = &state.identity;
  let project_role = repository.project_role(id, actor.user_id).await?.level();
  let organization_role = organization_level(
    state,
    actor,
    project["organization_id"]
      .as_i64()
      .ok_or_else(|| AppError::internal("invalid organization"))?,
  )
  .await?;
  Ok(project_role.max(organization_role))
}
pub async fn organization_level(
  state: &AppState,
  actor: &Principal,
  id: i64,
) -> Result<i64, AppError> {
  if actor.is_super_admin {
    return Ok(100);
  }
  Ok(
    state
      .identity
      .organization_role(id, actor.user_id)
      .await?
      .level(),
  )
}
pub async fn project(
  state: &AppState,
  headers: &HeaderMap,
  id: i64,
  minimum_role: i64,
) -> Result<Value, AppError> {
  let project = state
    .identity
    .active_project(id)
    .await?
    .map(project_record)
    .ok_or_else(|| AppError::not_found("project not found"))?;
  let public = project["visibility"] == "public";
  if minimum_role == 0 && public && !headers.contains_key("authorization") {
    return Ok(project);
  }
  let actor = actor(state, headers).await?;
  let level = level(state, &actor, &project).await?;
  let can_read = public || project["visibility"] == "internal" || level > 0;
  if (minimum_role == 0 && can_read) || (minimum_role > 0 && level >= minimum_role) {
    Ok(project)
  } else {
    Err(AppError::forbidden("insufficient project permissions"))
  }
}

pub(super) fn project_record(project: gity_entities::projects::Model) -> Value {
  serde_json::json!({
      "id": project.id,
      "organization_id": project.organization_id,
      "name": project.name,
      "path_key": project.path_key,
      "full_path": project.full_path,
      "visibility": project.visibility,
      "description": project.description,
      "default_branch": project.default_branch,
      "status": project.status,
      "deleted_at": project.deleted_at.map(timestamp),
      "created_at": timestamp(project.created_at),
      "updated_at": timestamp(project.updated_at),
  })
}

fn timestamp(value: chrono::NaiveDateTime) -> String {
  value
    .and_utc()
    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
