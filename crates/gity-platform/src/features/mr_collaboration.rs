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
  let collab = &s.collaboration;
  let row = model_value(collab.merge_request(p, i).await?);
  let (key, list) = match collection {
    MergeCollection::Comments => ("comments", values(collab.merge_comments(p, i).await?)),
    MergeCollection::Participants => (
      "participants",
      values(collab.merge_participants(p, i).await?),
    ),
    MergeCollection::Approvals => ("approvals", values(collab.merge_approvals(p, i).await?)),
  };
  Ok(json!({"merge_request":super::merge_requests::view(row),key:list}))
}
fn values<T: serde::Serialize>(rows: Vec<T>) -> Vec<Value> {
  rows.into_iter().map(model_value).collect()
}
/// Review activity reaches the author as a project todo, never the actor itself.
async fn tell(
  s: &AppState,
  p: i64,
  iid: i64,
  user: i64,
  kind: &str,
  row: &Value,
  target: i64,
) -> Result<(), AppError> {
  let title = match kind {
    "merge_request_commented" => format!("New comment on merge request !{iid}"),
    "merge_request_approved" => format!("Merge request !{iid} approved"),
    "merge_request_unapproved" => format!("Merge request !{iid} approval withdrawn"),
    "merge_request_review_requested" => format!("Review requested on merge request !{iid}"),
    _ => format!("Assigned to merge request !{iid}"),
  };
  let comment = kind == "merge_request_commented";
  let target_type = ["merge_request", "merge_request_comment"][usize::from(comment)];
  super::todos::notify(
    s,
    Notification {
      project: p,
      user,
      kind,
      target_type,
      target,
      title: &title,
      summary: string(row, "title"),
      suffix: "?tab=merge-requests",
    },
  )
  .await
}
/// Accepts the two historical spellings of a comment body.
fn comment_text(b: &Value) -> Result<String, AppError> {
  if string(b, "body").is_empty() {
    required(b, "content")
  } else {
    required(b, "body")
  }
}
/// Authors edit their own comments; other users need merge request write access.
async fn editable(s: &AppState, h: &HeaderMap, p: i64, i: i64, c: i64) -> Result<(), AppError> {
  let row = s.collaboration.merge_comment(p, i, c).await?;
  let review = ProjectAction::MergeRequestComment;
  let write = ProjectAction::MergeRequestWrite;
  super::issue_collaboration::authorize_comment(s, h, p, row.author_user_id, review, write).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post,path="/{id}/merge-requests/{iid}/comments",tag="mr collaboration",
    params(api::IidPath),
    request_body(content=crate::openapi::discussion_schemas::CreateComment),
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
  let command =
    super::discussions::merge_comment(&b, actor.user_id, number(&row, "id"), comment_text(&b)?)?;
  let created = model_value(s.collaboration.create_merge_comment(p, i, command).await?);
  let author = number(&row, "author_user_id");
  if author != actor.user_id {
    let target = number(&created, "id");
    tell(&s, p, i, author, "merge_request_commented", &row, target).await?;
  }
  response(child(&s, p, i, MergeCollection::Comments).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/merge-requests/{iid}/comments/{comment}",tag="mr collaboration",
    params(api::IidPath,("comment"=i64,Path,description="Comment identifier")),
    request_body(content=crate::openapi::issue_schemas::CreateComment),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeComments>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn update_comment(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, c)): Path<(i64, i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  editable(&s, &h, p, i, c).await?;
  let body = comment_text(&b)?;
  s.collaboration
    .update_merge_comment(p, i, c, MergeCommentUpdate { body })
    .await?;
  response(child(&s, p, i, MergeCollection::Comments).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete,path="/{id}/merge-requests/{iid}/comments/{comment}",tag="mr collaboration",
    params(api::IidPath,("comment"=i64,Path,description="Comment identifier")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn delete_comment(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, c)): Path<(i64, i64, i64)>,
) -> Reply {
  editable(&s, &h, p, i, c).await?;
  s.collaboration.delete_merge_comment(p, i, c).await?;
  response(json!({"ok":true}))
}
pub(super) async fn set_role(
  s: AppState,
  h: HeaderMap,
  p: i64,
  i: i64,
  b: Value,
  role: &str,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestWrite)
    .await?
    .actor()?;
  let _guard = lock(p).await;
  let row = model_value(&s.collaboration.merge_request(p, i).await?);
  let ids = users(&s, &b["user_ids"]).await?;
  s.collaboration
    .replace_participants(p, i, role.to_owned(), ids.clone())
    .await?;
  let kinds = ["merge_request_assigned", "merge_request_review_requested"];
  let kind = kinds[usize::from(role == "reviewer")];
  for user in ids.into_iter().filter(|user| *user != actor.user_id) {
    tell(&s, p, i, user, kind, &row, number(&row, "id")).await?;
  }
  response(child(&s, p, i, MergeCollection::Participants).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
  let author = number(&row, "author_user_id");
  if author != actor.user_id {
    let kinds = ["merge_request_unapproved", "merge_request_approved"];
    let kind = kinds[usize::from(yes)];
    tell(&s, p, i, author, kind, &row, number(&row, "id")).await?;
  }
  response(child(&s, p, i, MergeCollection::Approvals).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
