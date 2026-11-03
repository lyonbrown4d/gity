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
#[gity_api_macros::application_operation(prefix = "/api/v1")]
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
  let rows = s
    .collaboration
    .todos(actor.user_id, state, limit as u64)
    .await?;
  let rows = super::resource_visibility::filter_todos(&s, &h, rows).await?;
  response(json!(rows.into_iter().map(model_value).collect::<Vec<_>>()))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/todos/{id}/done",tag="todos",
    params(api::TodoPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::TodoDone>),api::Errors),
    security(("bearer"=[]))
)]
async fn done(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  s.collaboration.complete_todo(actor.user_id, id).await?;
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
  let result = s
    .collaboration
    .notify(gity_collaboration::Notification {
      project,
      user,
      kind: kind.to_owned(),
      target_type: target_type.to_owned(),
      target: target.to_string(),
      title: title.to_owned(),
      summary: summary.to_owned(),
      suffix: suffix.to_owned(),
    })
    .await;
  if let Err(error) = result {
    tracing::warn!(project_id=project,user_id=user,error=%error,"collaboration todo creation failed");
  }
  Ok(())
}
