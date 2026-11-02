//! Project label API.
//!
//! Labels are project-scoped definitions; issues and merge requests reference
//! them through polymorphic links instead of inlining a name and colour.
use super::*;
use crate::openapi::{label_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
use gity_entities::project_labels as labels;

/// Default colour used when a request omits one, matching the schema default.
pub(super) const DEFAULT_COLOR: &str = "#428BCA";

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(update, remove));
  utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared)
}

fn timestamp(value: chrono::NaiveDateTime) -> String {
  value
    .and_utc()
    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub(super) fn view(model: &labels::Model) -> Value {
  json!({
    "id": model.id,
    "project_id": model.project_id,
    "title": model.title,
    "color": model.color,
    "description": model.description,
    "created_at": timestamp(model.created_at),
    "updated_at": timestamp(model.updated_at),
  })
}

/// Label JSON as attached to one issue, keyed by the project-local issue number.
pub(super) fn issue_view(model: &labels::Model, issue_iid: i64) -> Value {
  json!({
    "id": model.id,
    "issue_id": issue_iid.to_string(),
    "name": model.title,
    "color": model.color,
    "description": model.description,
    "created_at": timestamp(model.created_at),
    "updated_at": timestamp(model.updated_at),
  })
}

fn color(body: &Value) -> String {
  let value = string(body, "color").trim();
  if value.is_empty() {
    DEFAULT_COLOR.to_owned()
  } else {
    value.to_owned()
  }
}

fn description(body: &Value) -> Option<String> {
  let value = string(body, "description").trim();
  (!value.is_empty()).then(|| value.to_owned())
}

async fn find(s: &AppState, project: i64, title: &str) -> Result<Option<labels::Model>, AppError> {
  s.collaboration.named_label(project, title).await
}
async fn load(s: &AppState, project: i64, label: i64) -> Result<labels::Model, AppError> {
  s.collaboration.label(project, label).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/labels",tag="labels",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ProjectLabel>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let rows = s.collaboration.labels(p).await?;
  response(json!(rows.iter().map(view).collect::<Vec<_>>()))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post,path="/{id}/labels",tag="labels",
    params(api::ProjectPath),
    request_body(content=spec::CreateLabel),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ProjectLabel>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  let title = required(&b, "title")?;
  if find(&s, p, &title).await?.is_some() {
    return Err(AppError::bad_request("label title already exists"));
  }
  let model = s
    .collaboration
    .create_label(
      p,
      LabelCreate {
        title,
        color: color(&b),
        description: description(&b),
      },
    )
    .await?;
  response(view(&model))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/labels/{label_id}",tag="labels",
    params(api::ProjectPath,("label_id"=i64,Path,description="Project label id")),
    request_body(content=spec::UpdateLabel),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ProjectLabel>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, label)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  let current = load(&s, p, label).await?;
  let title = match b.get("title").filter(|value| !value.is_null()) {
    Some(_) => {
      let title = required(&b, "title")?;
      if title != current.title && find(&s, p, &title).await?.is_some() {
        return Err(AppError::bad_request("label title already exists"));
      }
      Some(title)
    }
    None => None,
  };
  let color = match b.get("color").filter(|value| !value.is_null()) {
    Some(_) => {
      let value = string(&b, "color").trim();
      if value.is_empty() {
        return Err(AppError::bad_request("color must not be empty"));
      }
      Some(value.to_owned())
    }
    None => None,
  };
  let description = b.get("description").map(|_| description(&b));
  let model = s
    .collaboration
    .update_label(
      p,
      label,
      LabelChanges {
        title: title.map_or(Change::Unchanged, Change::Set),
        color: color.map_or(Change::Unchanged, Change::Set),
        description: description.map_or(Change::Unchanged, Change::Set),
      },
    )
    .await?;
  response(view(&model))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete,path="/{id}/labels/{label_id}",tag="labels",
    params(api::ProjectPath,("label_id"=i64,Path,description="Project label id")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn remove(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, label)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  s.collaboration.delete_label(p, label).await?;
  response(json!({"status":"deleted"}))
}
