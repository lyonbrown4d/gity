use super::merge_requests::lock;
use super::*;
use crate::openapi::{merge_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
pub(super) async fn child(
  s: &AppState,
  p: i64,
  i: i64,
  table: &str,
  key: &str,
) -> Result<Value, AppError> {
  let row = scoped(s, "project_merge_requests", p, i, true).await?;
  let list = s
    .db
    .all(
      &format!("SELECT * FROM {table} WHERE merge_request_id = ? ORDER BY created_at,id"),
      vec![number(&row, "id").into()],
    )
    .await?;
  Ok(json!({"merge_request":row,key:list}))
}
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/comments",tag="mr collaboration",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeComments>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn comments(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(child(&s, p, i, "project_merge_request_comments", "comments").await?)
}
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/participants",tag="mr collaboration",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Participants>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn participants(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(
    child(
      &s,
      p,
      i,
      "project_merge_request_participants",
      "participants",
    )
    .await?,
  )
}
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/approvals",tag="mr collaboration",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Approvals>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn approvals(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(child(&s, p, i, "project_merge_request_approvals", "approvals").await?)
}
#[utoipa::path(
    post,path="/{id}/merge-requests/{iid}/comments",tag="mr collaboration",
    params(api::IidPath),
    request_body(content=crate::openapi::issue_schemas::CreateComment),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeComments>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn comment(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestComment)
    .await?
    .actor()?;
  let row = scoped(&s, "project_merge_requests", p, i, true).await?;
  let text = if string(&b, "body").is_empty() {
    required(&b, "content")?
  } else {
    required(&b, "body")?
  };
  s.db
    .insert(
      "project_merge_request_comments",
      fields(
        json!({"merge_request_id":number(&row,"id"),"author_user_id":actor.user_id,"body":text}),
      ),
    )
    .await?;
  response(child(&s, p, i, "project_merge_request_comments", "comments").await?)
}
pub(super) async fn set_role(
  s: AppState,
  h: HeaderMap,
  p: i64,
  i: i64,
  b: Value,
  role: &str,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestWrite).await?;
  let _guard = lock(p).await;
  let row = scoped(&s, "project_merge_requests", p, i, true).await?;
  let ids = users(&s, &b["user_ids"]).await?;
  replace(
    &s,
    "project_merge_request_participants",
    "merge_request_id",
    number(&row, "id"),
    Some(role),
    ids
      .into_iter()
      .map(|u| fields(json!({"merge_request_id":number(&row,"id"),"user_id":u,"role":role})))
      .collect(),
  )
  .await?;
  response(
    child(
      &s,
      p,
      i,
      "project_merge_request_participants",
      "participants",
    )
    .await?,
  )
}
#[utoipa::path(
    patch,path="/{id}/merge-requests/{iid}/reviewers",tag="mr collaboration",
    params(api::IidPath),
    request_body(content=api::UserIds),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Participants>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn reviewers(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  set_role(s, h, p, i, b, "reviewer").await
}
#[utoipa::path(
    patch,path="/{id}/merge-requests/{iid}/assignees",tag="mr collaboration",
    params(api::IidPath),
    request_body(content=api::UserIds),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Participants>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn assignees(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  set_role(s, h, p, i, b, "assignee").await
}
pub(super) async fn approval(s: AppState, h: HeaderMap, p: i64, i: i64, yes: bool) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestWrite)
    .await?
    .actor()?;
  let _guard = lock(p).await;
  let row = scoped(&s, "project_merge_requests", p, i, true).await?;
  if string(&row, "state") != "opened" {
    return Err(AppError::conflict("merge request is not opened"));
  }
  let id = number(&row, "id");
  let existing = s
    .db
    .one(
      "SELECT id FROM project_merge_request_approvals WHERE merge_request_id = ? AND user_id = ?",
      vec![id.into(), actor.user_id.into()],
    )
    .await?;
  if yes && existing.is_none() {
    s.db
      .insert(
        "project_merge_request_approvals",
        fields(json!({"merge_request_id":id,"user_id":actor.user_id})),
      )
      .await?;
  } else if !yes && let Some(row) = existing {
    s.db
      .delete("project_merge_request_approvals", number(&row, "id"))
      .await?;
  }
  response(child(&s, p, i, "project_merge_request_approvals", "approvals").await?)
}
#[utoipa::path(
    post,path="/{id}/merge-requests/{iid}/approve",tag="mr collaboration",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Approvals>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn approve(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  approval(s, h, p, i, true).await
}
#[utoipa::path(
    post,path="/{id}/merge-requests/{iid}/unapprove",tag="mr collaboration",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Approvals>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn unapprove(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  approval(s, h, p, i, false).await
}
