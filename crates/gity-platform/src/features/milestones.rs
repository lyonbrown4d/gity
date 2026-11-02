//! Project milestone API.
use super::*;
use crate::openapi::{milestone_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
use gity_entities::{project_issues, project_milestones as entity};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, sea_query::Expr,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().nest(
    "/projects",
    utoipa_axum::router::OpenApiRouter::new()
      .routes(utoipa_axum::routes!(list, create))
      .routes(utoipa_axum::routes!(detail, update, remove)),
  )
}
#[utoipa::path(
    get,path="/{id}/milestones",tag="milestones",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Milestone>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let rows = entity::Entity::find()
    .filter(entity::Column::ProjectId.eq(p))
    .order_by_asc(entity::Column::Iid)
    .all(&s.db.connection)
    .await?;
  response(json!(rows.into_iter().map(view).collect::<Vec<_>>()))
}
#[utoipa::path(
    get,path="/{id}/milestones/{iid}",tag="milestones",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Milestone>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(view(find(&s, p, i).await?))
}
#[utoipa::path(
    post,path="/{id}/milestones",tag="milestones",
    params(api::ProjectPath),
    request_body(content=spec::CreateMilestone),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Milestone>),api::Errors),
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
  let state = match b.get("state") {
    Some(value) => state(value)?,
    None => "active".to_owned(),
  };
  let iid = s.db.next_iid(p, "milestone").await?;
  let timestamp = chrono::Utc::now().naive_utc();
  let row = entity::ActiveModel {
    project_id: Set(p),
    iid: Set(iid),
    title: Set(title),
    description: Set(text(&b)),
    state: Set(state),
    start_date: Set(day(b.get("start_date"))?),
    due_date: Set(day(b.get("due_date"))?),
    created_at: Set(timestamp),
    updated_at: Set(timestamp),
    ..Default::default()
  }
  .insert(&s.db.connection)
  .await?;
  response(view(row))
}
#[utoipa::path(
    patch,path="/{id}/milestones/{iid}",tag="milestones",
    params(api::IidPath),
    request_body(content=spec::UpdateMilestone),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Milestone>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  let current = find(&s, p, i).await?;
  let mut change = entity::ActiveModel {
    id: Set(current.id),
    updated_at: Set(chrono::Utc::now().naive_utc()),
    ..Default::default()
  };
  if b.get("title").is_some() {
    change.title = Set(required(&b, "title")?);
  }
  if b.get("description").is_some() {
    change.description = Set(text(&b));
  }
  if let Some(value) = b.get("state") {
    change.state = Set(state(value)?);
  }
  if b.get("start_date").is_some() {
    change.start_date = Set(day(b.get("start_date"))?);
  }
  if b.get("due_date").is_some() {
    change.due_date = Set(day(b.get("due_date"))?);
  }
  let row = change.update(&s.db.connection).await?;
  response(view(row))
}
#[utoipa::path(
    delete,path="/{id}/milestones/{iid}",tag="milestones",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn remove(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  let current = find(&s, p, i).await?;
  // Issues keep their history; only the milestone reference is detached.
  project_issues::Entity::update_many()
    .col_expr(
      project_issues::Column::MilestoneId,
      Expr::value(Option::<i64>::None),
    )
    .filter(project_issues::Column::MilestoneId.eq(current.id))
    .exec(&s.db.connection)
    .await?;
  entity::Entity::delete_by_id(current.id)
    .exec(&s.db.connection)
    .await?;
  response(json!({"status":"deleted"}))
}
async fn find(s: &AppState, project: i64, iid: i64) -> Result<entity::Model, AppError> {
  entity::Entity::find()
    .filter(entity::Column::ProjectId.eq(project))
    .filter(entity::Column::Iid.eq(iid))
    .one(&s.db.connection)
    .await?
    .ok_or_else(|| AppError::not_found("milestone not found"))
}
fn view(row: entity::Model) -> Value {
  json!({
    "id": row.id.to_string(),
    "project_id": row.project_id.to_string(),
    "iid": row.iid,
    "title": row.title,
    "description": row.description,
    "state": row.state,
    "start_date": row.start_date.map(calendar),
    "due_date": row.due_date.map(calendar),
    "created_at": stamp(row.created_at),
    "updated_at": stamp(row.updated_at),
  })
}
fn stamp(value: chrono::NaiveDateTime) -> String {
  value
    .and_utc()
    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
fn calendar(value: chrono::NaiveDate) -> String {
  value.format("%Y-%m-%d").to_string()
}
fn state(value: &Value) -> Result<String, AppError> {
  match value.as_str() {
    Some("active") => Ok("active".to_owned()),
    Some("closed") => Ok("closed".to_owned()),
    _ => Err(AppError::bad_request(
      "milestone state must be active or closed",
    )),
  }
}
fn text(b: &Value) -> Option<String> {
  b.get("description")
    .and_then(Value::as_str)
    .map(str::to_owned)
}
fn day(value: Option<&Value>) -> Result<Option<chrono::NaiveDate>, AppError> {
  match value {
    None | Some(Value::Null) => Ok(None),
    Some(Value::String(raw)) => {
      let raw = raw.trim();
      if raw.is_empty() {
        return Ok(None);
      }
      chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| AppError::bad_request("milestone dates must use YYYY-MM-DD"))
    }
    Some(_) => Err(AppError::bad_request("milestone dates must use YYYY-MM-DD")),
  }
}
