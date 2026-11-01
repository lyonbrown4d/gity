use super::merge_requests::{git_error, repository};
use super::tags::tag;
use super::*;
use crate::openapi::{release_schemas as spec, schemas as api};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
};
use std::collections::HashMap;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    let shared = utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            super::tags::tags,
            super::tags::create_tag,
            super::tags::delete_tag
        ))
        .routes(utoipa_axum::routes!(list, create))
        .routes(utoipa_axum::routes!(detail, update, remove))
        .routes(utoipa_axum::routes!(link))
        .routes(utoipa_axum::routes!(unlink));
    utoipa_axum::router::OpenApiRouter::new()
        .nest("/projects", shared.clone())
        .nest("/repos", shared)
}
async fn value(s: &AppState, p: &Value, row: Value) -> Result<Value, AppError> {
    let links =
        s.db.all(
            "SELECT * FROM project_release_links WHERE project_release_id = ? ORDER BY id",
            vec![number(&row, "id").into()],
        )
        .await?;
    let tag = tag(s, p, string(&row, "tag_name")).await?;
    Ok(json!({"release":row,"links":links,"tag":tag}))
}
#[utoipa::path(
    get,path="/{id}/releases",tag="releases",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ReleaseDetail>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
    let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
        .await?
        .project;
    let mut out = Vec::new();
    for row in
        s.db.all(
            "SELECT * FROM project_releases WHERE project_id = ? ORDER BY released_at DESC,id DESC",
            vec![p.into()],
        )
        .await?
    {
        out.push(value(&s, &project, row).await?);
    }
    response(json!(out))
}
#[utoipa::path(
    get,path="/{id}/releases/{release}",tag="releases",
    params(api::ReleasePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseDetail>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, r)): Path<(i64, i64)>) -> Reply {
    let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
        .await?
        .project;
    let row = scoped(&s, "project_releases", p, r, false).await?;
    response(value(&s, &project, row).await?)
}
fn time(b: &Value) -> Result<String, AppError> {
    let raw = string(b, "released_at");
    if raw.is_empty() {
        Ok(now())
    } else {
        chrono::DateTime::parse_from_rfc3339(raw)
            .map(|v| v.to_utc().to_rfc3339())
            .map_err(|_| AppError::bad_request("invalid released_at"))
    }
}
#[utoipa::path(
    post,path="/{id}/releases",tag="releases",
    params(api::ProjectPath),
    request_body(content=spec::CreateRelease),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseDetail>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(p): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    let authorization =
        crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
    let actor = authorization.actor()?;
    let project = authorization.project;
    let _guard = super::merge_requests::lock(p).await;
    let name = required(&b, "tag_name")?;
    let released = time(&b)?;
    if s.db
        .one(
            "SELECT id FROM project_releases WHERE project_id = ? AND tag_name = ?",
            vec![p.into(), name.clone().into()],
        )
        .await?
        .is_some()
    {
        return Err(AppError::conflict("release already exists for tag"));
    }
    if tag(&s, &project, &name).await?.is_none() {
        if b["create_tag"] != true {
            return Err(AppError::not_found("release tag not found"));
        }
        let source = if string(&b, "source_ref").is_empty() {
            string(&project, "default_branch")
        } else {
            string(&b, "source_ref")
        };
        s.git
            .create_tag(&repository(&project), &name, source)
            .await
            .map_err(git_error)?;
    }
    let title = if string(&b, "name").trim().is_empty() {
        name.clone()
    } else {
        string(&b, "name").trim().to_owned()
    };
    let row=s.db.insert("project_releases",fields(json!({"project_id":p,"tag_name":name,"name":title,"description":string(&b,"description").trim(),"created_by_user_id":actor.user_id,"released_at":released}))).await?;
    response(value(&s, &project, row).await?)
}
#[utoipa::path(
    patch,path="/{id}/releases/{release}",tag="releases",
    params(api::ReleasePath),
    request_body(content=spec::UpdateRelease),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseDetail>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, r)): Path<(i64, i64)>,
    Json(b): Json<Value>,
) -> Reply {
    let project = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
        .await?
        .project;
    scoped(&s, "project_releases", p, r, false).await?;
    let mut change = patch(&b, &["name", "description"]);
    if b.get("released_at").is_some() {
        change.insert("released_at".into(), json!(time(&b)?));
    }
    let row = s.db.update("project_releases", r, change).await?;
    response(value(&s, &project, row).await?)
}
#[utoipa::path(
    delete,path="/{id}/releases/{release}",tag="releases",
    params(api::ReleasePath,("delete_tag"=Option<bool>,Query)),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn remove(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, r)): Path<(i64, i64)>,
    Query(q): Query<HashMap<String, String>>,
) -> Reply {
    let project = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
        .await?
        .project;
    let row = scoped(&s, "project_releases", p, r, false).await?;
    if q.get("delete_tag").is_some_and(|v| v == "true")
        && tag(&s, &project, string(&row, "tag_name")).await?.is_some()
    {
        s.git
            .delete_tag(&repository(&project), string(&row, "tag_name"))
            .await
            .map_err(git_error)?;
    }
    s.db.delete("project_releases", r).await?;
    response(json!({"status":"deleted"}))
}
#[utoipa::path(
    post,path="/{id}/releases/{release}/links",tag="releases",
    params(api::ReleasePath),
    request_body(content=spec::CreateReleaseLink),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseLink>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn link(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, r)): Path<(i64, i64)>,
    Json(b): Json<Value>,
) -> Reply {
    crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
    scoped(&s, "project_releases", p, r, false).await?;
    let name = required(&b, "name")?;
    let url = required(&b, "url")?;
    let parsed = reqwest::Url::parse(&url)
        .map_err(|_| AppError::bad_request("release link url is invalid"))?;
    if !["http", "https"].contains(&parsed.scheme()) || parsed.host_str().is_none() {
        return Err(AppError::bad_request("release link url is invalid"));
    }
    response(s.db.insert("project_release_links",fields(json!({"project_release_id":r,"name":name,"url":url,"link_type":if string(&b,"link_type").is_empty(){"other"}else{string(&b,"link_type")}}))).await?)
}
#[utoipa::path(
    delete,path="/{id}/releases/{release}/links/{link}",tag="releases",
    params(api::LinkPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn unlink(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, r, l)): Path<(i64, i64, i64)>,
) -> Reply {
    crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
    scoped(&s, "project_releases", p, r, false).await?;
    if s.db
        .one(
            "SELECT id FROM project_release_links WHERE id = ? AND project_release_id = ?",
            vec![l.into(), r.into()],
        )
        .await?
        .is_none()
    {
        return Err(AppError::not_found("release link not found"));
    }
    s.db.delete("project_release_links", l).await?;
    response(json!({"status":"deleted"}))
}
