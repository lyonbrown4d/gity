use super::merge_requests::lock;
use super::*;
use crate::openapi::{merge_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
#[derive(Clone, Copy)]
pub(super) enum MergeCollection {
  Comments,
  Participants,
  Approvals,
}
pub(super) async fn child(
  s: &AppState,
  p: i64,
  i: i64,
  collection: MergeCollection,
) -> Result<Value, AppError> {
  let row = model_value(s.collaboration.merge_request(p, i).await?);
  let (key, list) = match collection {
    MergeCollection::Comments => (
      "comments",
      s.collaboration
        .merge_comments(p, i)
        .await?
        .into_iter()
        .map(model_value)
        .collect::<Vec<_>>(),
    ),
    MergeCollection::Participants => (
      "participants",
      s.collaboration
        .merge_participants(p, i)
        .await?
        .into_iter()
        .map(model_value)
        .collect::<Vec<_>>(),
    ),
    MergeCollection::Approvals => (
      "approvals",
      s.collaboration
        .merge_approvals(p, i)
        .await?
        .into_iter()
        .map(model_value)
        .collect::<Vec<_>>(),
    ),
  };
  Ok(json!({"merge_request":super::merge_requests::view(row),key:list}))
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
  response(child(&s, p, i, MergeCollection::Comments).await?)
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
  response(child(&s, p, i, MergeCollection::Participants).await?)
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
  response(child(&s, p, i, MergeCollection::Approvals).await?)
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
  let row = model_value(&s.collaboration.merge_request(p, i).await?);
  let text = if string(&b, "body").is_empty() {
    required(&b, "content")?
  } else {
    required(&b, "body")?
  };
  s.collaboration
    .create_merge_comment(
      p,
      i,
      MergeCommentCreate {
        merge_request_id: number(&row, "id"),
        author_user_id: actor.user_id,
        body: text,
      },
    )
    .await?;
  response(child(&s, p, i, MergeCollection::Comments).await?)
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
  let ids = users(&s, &b["user_ids"]).await?;
  s.collaboration
    .replace_participants(p, i, role.to_owned(), ids)
    .await?;
  response(child(&s, p, i, MergeCollection::Participants).await?)
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
  let row = model_value(&s.collaboration.merge_request(p, i).await?);
  if string(&row, "state") != "opened" {
    return Err(AppError::conflict("merge request is not opened"));
  }
  s.collaboration.approve(p, i, actor.user_id, yes).await?;
  response(child(&s, p, i, MergeCollection::Approvals).await?)
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
