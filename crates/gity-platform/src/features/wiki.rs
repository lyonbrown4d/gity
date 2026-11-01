use super::*;
use crate::openapi::{schemas as api, wiki_schemas as spec};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail, update, remove));
  utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared)
}
fn slug(v: &str) -> Result<String, AppError> {
  let mut out = String::new();
  for c in deunicode::deunicode(v.trim()).to_lowercase().chars() {
    if c.is_alphanumeric() {
      out.push(c);
    } else if !out.ends_with('-') && !out.is_empty() {
      out.push('-');
    }
  }
  let out = out.trim_matches('-').to_owned();
  if out.is_empty() || out.len() > 160 {
    return Err(AppError::bad_request("invalid wiki page slug"));
  }
  Ok(out)
}
async fn page(s: &AppState, p: i64, key: &str) -> Result<Value, AppError> {
  s.db
    .one(
      "SELECT * FROM project_wiki_pages WHERE project_id = ? AND slug = ?",
      vec![p.into(), slug(key)?.into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("wiki page not found"))
}
#[utoipa::path(
    get,path="/{id}/wiki/pages",tag="wiki",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::WikiPage>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::WikiRead).await?;
  response(json!(
    s.db
      .all(
        "SELECT * FROM project_wiki_pages WHERE project_id = ? ORDER BY title,id",
        vec![p.into()]
      )
      .await?
  ))
}
#[utoipa::path(
    get,path="/{id}/wiki/pages/{slug}",tag="wiki",
    params(api::WikiPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::WikiPage>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, key)): Path<(i64, String)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::WikiRead).await?;
  response(page(&s, p, &key).await?)
}
#[utoipa::path(
    post,path="/{id}/wiki/pages",tag="wiki",
    params(api::ProjectPath),
    request_body(content=spec::CreateWikiPage),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::WikiPage>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::WikiWrite)
    .await?
    .actor()?;
  let title = required(&b, "title")?;
  let format = string(&b, "format").trim().to_lowercase();
  if !format.is_empty() && format != "markdown" {
    return Err(AppError::bad_request("wiki page format must be markdown"));
  }
  let key = slug(if string(&b, "slug").is_empty() {
    &title
  } else {
    string(&b, "slug")
  })?;
  if s
    .db
    .one(
      "SELECT id FROM project_wiki_pages WHERE project_id = ? AND slug = ?",
      vec![p.into(), key.clone().into()],
    )
    .await?
    .is_some()
  {
    return Err(AppError::conflict("wiki page already exists"));
  }
  response(s.db.insert("project_wiki_pages",fields(json!({"project_id":p,"slug":key,"title":title,"content":string(&b,"content"),"format":"markdown","author_user_id":actor.user_id,"last_edited_by_user_id":actor.user_id}))).await?)
}
#[utoipa::path(
    patch,path="/{id}/wiki/pages/{slug}",tag="wiki",
    params(api::WikiPath),
    request_body(content=spec::UpdateWikiPage),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::WikiPage>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, key)): Path<(i64, String)>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::WikiWrite)
    .await?
    .actor()?;
  let row = page(&s, p, &key).await?;
  if b.get("title").is_some() {
    required(&b, "title")?;
  }
  let mut change = patch(&b, &["title", "content"]);
  change.insert("last_edited_by_user_id".into(), json!(actor.user_id));
  response(
    s.db
      .update("project_wiki_pages", number(&row, "id"), change)
      .await?,
  )
}
#[utoipa::path(
    delete,path="/{id}/wiki/pages/{slug}",tag="wiki",
    params(api::WikiPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::WikiPage>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn remove(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, key)): Path<(i64, String)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::WikiWrite).await?;
  let row = page(&s, p, &key).await?;
  s.db
    .delete("project_wiki_pages", number(&row, "id"))
    .await?;
  response(row)
}
#[cfg(test)]
#[path = "wiki_tests.rs"]
mod tests;
