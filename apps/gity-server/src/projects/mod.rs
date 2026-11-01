use crate::openapi::{project_schemas as spec, schemas as api};
mod creation;
mod members;
mod namespace;
mod permission;
use permission::permissions;
mod views;
use views::{organization_view, project_view};
mod org_members;
mod org_membership;
mod organizations;
use crate::{AppState, access, auth};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use gity_core::{AppError, ok};
use organizations::*;
use serde_json::{Value, json};
use std::collections::HashMap;
type Reply = Result<Json<Value>, AppError>;
pub fn routes() -> Router<AppState> {
    documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    let mut router = utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list, creation::create))
        .routes(utoipa_axum::routes!(detail, update, remove))
        .routes(utoipa_axum::routes!(permission::permissions))
        .routes(utoipa_axum::routes!(
            organizations::orgs,
            organizations::create_org
        ))
        .routes(utoipa_axum::routes!(
            organizations::org,
            organizations::update_org,
            organizations::delete_org
        ))
        .routes(utoipa_axum::routes!(members::members, members::add_member))
        .routes(utoipa_axum::routes!(
            members::set_member,
            members::delete_member
        ))
        .routes(utoipa_axum::routes!(
            org_members::org_members,
            org_members::add_org_member
        ))
        .routes(utoipa_axum::routes!(
            org_members::set_org_member,
            org_members::delete_org_member
        ));
    for (source, target) in [
        ("/projects", "/repos"),
        ("/projects/{id}", "/repos/{id}"),
        ("/projects/{id}/permissions", "/repos/{id}/permissions"),
    ] {
        crate::openapi::alias(&mut router, source, target);
    }
    router
        .route("/repos", axum::routing::get(list).post(creation::create))
        .route(
            "/repos/{id}",
            axum::routing::get(detail).patch(update).delete(remove),
        )
        .route("/repos/{id}/permissions", axum::routing::get(permissions))
}
pub fn id(v: &Value) -> Result<i64, AppError> {
    v.as_i64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .filter(|n| *n > 0)
        .ok_or_else(|| AppError::bad_request("positive identifier required"))
}
pub fn slug(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_lowercase();
    if value.is_empty()
        || value.len() > 100
        || value == "."
        || value == ".."
        || value.ends_with(".git")
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
    {
        return Err(AppError::bad_request("invalid path_key"));
    }
    Ok(value)
}
fn visibility(b: &Value) -> Result<&str, AppError> {
    let value = b["visibility"].as_str().unwrap_or("private");
    if !["private", "internal", "public"].contains(&value) {
        return Err(AppError::bad_request("invalid visibility"));
    }
    Ok(value)
}
fn required<'a>(b: &'a Value, key: &str) -> Result<&'a str, AppError> {
    b[key]
        .as_str()
        .filter(|v| !v.trim().is_empty() && v.len() < 512)
        .ok_or_else(|| AppError::bad_request(format!("{key} required")))
}
fn fields(b: &Value, keys: &[&str]) -> serde_json::Map<String, Value> {
    keys.iter()
        .filter_map(|key| b.get(*key).map(|v| ((*key).into(), v.clone())))
        .collect()
}
#[utoipa::path(
    get,path="/projects",tag="projects",
    description="Authenticated users see public/internal projects and their memberships.",
    params(("organization_id"=Option<i64>,Query)),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Project>>),api::Errors),
    security(("bearer"=[]))
)]
async fn list(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Reply {
    let p = auth::principal(&s, &h).await?;
    let mut out = Vec::new();
    for item in
        s.db.all(
            "SELECT * FROM projects WHERE deleted_at IS NULL ORDER BY id DESC",
            vec![],
        )
        .await?
    {
        if let Some(org) = q.get("organization_id")
            && item["organization_id"].as_i64() != org.parse::<i64>().ok()
        {
            continue;
        }
        if p.is_super_admin
            || item["visibility"] != "private"
            || access::level(&s, &p, &item).await? > 0
        {
            out.push(project_view(&s, item));
        }
    }
    Ok(ok(out))
}
#[utoipa::path(
    get,path="/projects/{id}",tag="projects",
    description="Visibility and membership determine read access.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Project>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
    Ok(ok(project_view(
        &s,
        access::project_action(&s, &h, id, access::ProjectAction::Read)
            .await?
            .project,
    )))
}
#[utoipa::path(
    patch,path="/projects/{id}",tag="projects",
    description="Requires maintainer or higher.",
    params(api::ProjectPath),
    request_body=spec::UpdateProject,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Project>),api::Errors),
    security(("bearer"=[]))
)]
async fn update(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    access::project(&s, &h, id, 40).await?;
    if b.get("visibility").is_some() {
        visibility(&b)?;
    }
    Ok(ok(project_view(
        &s,
        s.db.update(
            "projects",
            id,
            fields(&b, &["name", "description", "visibility", "default_branch"]),
        )
        .await?,
    )))
}
#[utoipa::path(
    delete,path="/projects/{id}",tag="projects",
    description="Marks project pending deletion; owner and exact full_path confirmation required.",
    params(api::ProjectPath),request_body=spec::DeleteProject,
    responses((status=200,description="Operation succeeded",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),
    security(("bearer"=[]))
)]
async fn remove(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Reply {
    let project = access::project(&s, &h, id, 50).await?;
    if body["confirmation"].as_str().map(str::trim) != project["full_path"].as_str() {
        return Err(AppError::bad_request(
            "project delete confirmation does not match full path",
        ));
    }
    let actor = auth::principal(&s, &h).await?;
    s.db.update(
        "projects",
        id,
        json!({"deleted_at":gity_core::now(),"status":"pending_delete"})
            .as_object()
            .unwrap()
            .clone(),
    )
    .await?;
    crate::events::project_change(
        &s,
        &project,
        actor.user_id,
        crate::events::ProjectChange::Deleted,
    )
    .await?;
    Ok(ok(json!({"ok":true})))
}

#[cfg(test)]
mod permission_tests;

#[cfg(test)]
mod org_membership_tests;

#[cfg(test)]
mod namespace_tests;
