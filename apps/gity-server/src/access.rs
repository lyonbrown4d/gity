mod actions;
mod project_action;
use crate::{AppState, auth};
pub use actions::ProjectAction;
use axum::http::HeaderMap;
use gity_core::{AppError, Principal};
pub use project_action::{ProjectActor, ProjectAuthorization, project_action};
use serde_json::Value;

pub async fn actor(state: &AppState, headers: &HeaderMap) -> Result<Principal, AppError> {
    auth::principal(state, headers).await
}
pub fn role_level(role: &str) -> i64 {
    match role {
        "owner" => 50,
        "maintainer" => 40,
        "developer" => 30,
        "reporter" => 20,
        "guest" => 10,
        _ => 0,
    }
}
pub async fn level(state: &AppState, actor: &Principal, project: &Value) -> Result<i64, AppError> {
    if actor.is_super_admin {
        return Ok(100);
    }
    let id = project["id"]
        .as_i64()
        .ok_or_else(|| AppError::internal("invalid project"))?;
    let project_role = if let Some(member) = state
        .db
        .one(
            "SELECT role FROM project_members WHERE project_id=? AND user_id=?",
            vec![id.into(), actor.user_id.into()],
        )
        .await?
    {
        role_level(member["role"].as_str().unwrap_or_default())
    } else {
        0
    };
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
    Ok(state
        .db
        .one(
            "SELECT role FROM organization_members WHERE organization_id=? AND user_id=?",
            vec![id.into(), actor.user_id.into()],
        )
        .await?
        .map(|v| role_level(v["role"].as_str().unwrap_or_default()))
        .unwrap_or(0))
}
pub async fn project(
    state: &AppState,
    headers: &HeaderMap,
    id: i64,
    minimum_role: i64,
) -> Result<Value, AppError> {
    let project = state
        .db
        .one(
            "SELECT * FROM projects WHERE id=? AND deleted_at IS NULL",
            vec![id.into()],
        )
        .await?
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
