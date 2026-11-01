use super::merge_requests::{git_error, repository};
use super::*;
use crate::openapi::{release_schemas as spec, schemas as api};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};
use std::collections::HashMap;
pub(super) async fn tag(s: &AppState, p: &Value, name: &str) -> Result<Option<Value>, AppError> {
  Ok(
    s.git
      .list_tags(&repository(p))
      .await
      .map_err(git_error)?
      .as_array()
      .and_then(|v| v.iter().find(|v| string(v, "name") == name))
      .cloned(),
  )
}
#[utoipa::path(
    get,path="/{id}/repository/tags",tag="releases",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Tag>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn tags(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
    .await?
    .project;
  response(
    s.git
      .list_tags(&repository(&project))
      .await
      .map_err(git_error)?,
  )
}
#[utoipa::path(
    post,path="/{id}/repository/tags",tag="releases",
    params(api::ProjectPath),
    request_body(content=spec::CreateTag),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Tag>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn create_tag(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .project;
  let name = required(&b, "name")?;
  let source = if string(&b, "source_ref").is_empty() {
    string(&project, "default_branch")
  } else {
    string(&b, "source_ref")
  };
  s.git
    .create_tag(&repository(&project), &name, source)
    .await
    .map_err(git_error)?;
  response(tag(&s, &project, &name).await?.unwrap_or(Value::Null))
}
#[utoipa::path(
    delete,path="/{id}/repository/tags",tag="releases",
    params(api::ProjectPath,("name"=String,Query,description="Tag name")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn delete_tag(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .project;
  let name = q
    .get("name")
    .filter(|v| !v.trim().is_empty())
    .ok_or_else(|| AppError::bad_request("tag name is required"))?;
  s.git
    .delete_tag(&repository(&project), name)
    .await
    .map_err(git_error)?;
  response(json!({"status":"deleted"}))
}
