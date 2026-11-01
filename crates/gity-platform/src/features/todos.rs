use super::*;
use crate::openapi::{schemas as api, wiki_schemas as spec};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};
use std::collections::HashMap;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list))
    .routes(utoipa_axum::routes!(done))
}
#[utoipa::path(
    get,path="/todos",tag="todos",
    params(("state"=Option<String>,Query,description="pending or done"),("limit"=Option<i64>,Query,description="1–200; defaults to 50")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Todo>>),api::Errors),
    security(("bearer"=[]))
)]
async fn list(
  State(s): State<AppState>,
  h: HeaderMap,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  let state = q.get("state").map(String::as_str).unwrap_or("").trim();
  if !["", "pending", "done"].contains(&state) {
    return Err(AppError::bad_request("unsupported todo state"));
  }
  let limit = q
    .get("limit")
    .and_then(|v| v.parse::<i64>().ok())
    .filter(|v| *v > 0 && *v <= 200)
    .unwrap_or(50);
  response(json!(s.db.all("SELECT * FROM project_todos WHERE user_id = ? AND (? = '' OR state = ?) ORDER BY created_at DESC, id DESC LIMIT ?",vec![actor.user_id.into(),state.into(),state.into(),limit.into()]).await?))
}
#[utoipa::path(
    post,path="/todos/{id}/done",tag="todos",
    params(api::TodoPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::TodoDone>),api::Errors),
    security(("bearer"=[]))
)]
async fn done(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  let row = s
    .db
    .one(
      "SELECT id FROM project_todos WHERE id = ? AND user_id = ?",
      vec![id.into(), actor.user_id.into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("todo not found"))?;
  s.db
    .update(
      "project_todos",
      number(&row, "id"),
      fields(json!({"state":"done","done_at":now(),"updated_at":now()})),
    )
    .await?;
  response(json!({"id":id,"state":"done"}))
}
pub(crate) struct Notification<'a, T> {
  pub project: i64,
  pub user: i64,
  pub kind: &'a str,
  pub target_type: &'a str,
  pub target: T,
  pub title: &'a str,
  pub summary: &'a str,
  pub suffix: &'a str,
}
pub(crate) async fn notify(
  s: &AppState,
  notification: Notification<'_, impl ToString + Send>,
) -> Result<(), AppError> {
  let Notification {
    project,
    user,
    kind,
    target_type,
    target,
    title,
    summary,
    suffix,
  } = notification;
  let result = async {
        let row = s.db.one("SELECT organization_id FROM projects WHERE id = ?",vec![project.into()])
            .await?.ok_or_else(||AppError::not_found("project not found"))?;
        let org = number(&row,"organization_id");
        let conflict = if s.db.backend()==sea_orm::DatabaseBackend::MySql {
            "ON DUPLICATE KEY UPDATE id=id"
        } else {
            "ON CONFLICT(user_id,kind,target_type,target_id) DO NOTHING"
        };
        let columns="id,user_id,organization_id,project_id,kind,state,target_type,target_id,title,summary,action_url,created_at,updated_at";
        let sql=format!("INSERT INTO project_todos({columns}) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?) {conflict}");
        let args=vec![
            gity_core::new_id().into(), user.into(), org.into(), project.into(), kind.into(),
            "pending".into(), target_type.into(), target.to_string().into(), title.into(),
            summary.into(), format!("/app/projects/{org}/{project}{suffix}").into(), now().into(), now().into()
        ];
        s.db.execute(&sql,args).await?;
        Ok::<(),AppError>(())
    }.await;
  if let Err(error) = result {
    tracing::warn!(project_id=project,user_id=user,error=%error,"collaboration todo creation failed");
  }
  Ok(())
}
