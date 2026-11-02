//! Discussion threads: listing, resolving and the attach fields on comments.
//!
//! Threads are created implicitly: a comment without `discussion_id` opens one,
//! so the plain comment endpoints stay the only way to start a conversation.
use super::*;
use crate::openapi::{discussion_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let router = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(issue_threads))
    .routes(utoipa_axum::routes!(merge_threads))
    .routes(utoipa_axum::routes!(issue_resolve))
    .routes(utoipa_axum::routes!(issue_unresolve))
    .routes(utoipa_axum::routes!(merge_resolve))
    .routes(utoipa_axum::routes!(merge_unresolve));
  utoipa_axum::router::OpenApiRouter::new().nest("/projects", router)
}

/// Thread rows carry their comments; the view keeps the documented shape.
fn thread(discussion: Value, comments: Vec<Value>) -> Value {
  json!({
    "id": discussion["id"],
    "resolved": discussion["resolved"],
    "resolved_by_user_id": discussion["resolved_by_user_id"],
    "comments": comments,
    "created_at": discussion["created_at"],
    "updated_at": discussion["updated_at"],
  })
}

/// Stored anchors are JSON text; the API returns the parsed object.
fn comment(row: Value) -> Value {
  let position = row["position"]
    .as_str()
    .and_then(|text| serde_json::from_str::<Value>(text).ok())
    .unwrap_or(Value::Null);
  json!({
    "id": row["id"],
    "discussion_id": row["discussion_id"],
    "author_user_id": row["author_user_id"],
    "body": row["body"],
    "position": position,
    "created_at": row["created_at"],
    "updated_at": row["updated_at"],
  })
}

async fn threads(
  s: &AppState,
  target: DiscussionTarget,
  project: i64,
  iid: i64,
) -> Result<Vec<Value>, AppError> {
  Ok(match target {
    DiscussionTarget::Issue => s
      .collaboration
      .issue_discussions(project, iid)
      .await?
      .into_iter()
      .map(|(discussion, comments)| thread(model_value(discussion), rows(comments)))
      .collect(),
    DiscussionTarget::MergeRequest => s
      .collaboration
      .merge_discussions(project, iid)
      .await?
      .into_iter()
      .map(|(discussion, comments)| thread(model_value(discussion), rows(comments)))
      .collect(),
  })
}

fn rows<T: serde::Serialize>(comments: Vec<T>) -> Vec<Value> {
  comments.into_iter().map(model_value).map(comment).collect()
}

async fn one(
  s: &AppState,
  target: DiscussionTarget,
  project: i64,
  iid: i64,
  discussion: i64,
) -> Result<Value, AppError> {
  Ok(match target {
    DiscussionTarget::Issue => {
      let (row, comments) = s
        .collaboration
        .issue_discussion(project, iid, discussion)
        .await?;
      thread(model_value(row), rows(comments))
    }
    DiscussionTarget::MergeRequest => {
      let (row, comments) = s
        .collaboration
        .merge_discussion(project, iid, discussion)
        .await?;
      thread(model_value(row), rows(comments))
    }
  })
}

/// Readers see threads; only the thread author or a writer may resolve them.
async fn resolve(
  s: &AppState,
  h: &HeaderMap,
  project: i64,
  target: DiscussionTarget,
  iid: i64,
  discussion: i64,
  resolved: bool,
) -> Reply {
  let authorization = crate::access::project_action(s, h, project, ProjectAction::Read).await?;
  let actor = authorization.actor()?;
  let can_write = authorization.allows(ProjectAction::IssueWrite)
    || authorization.allows(ProjectAction::MergeRequestWrite);
  s.collaboration
    .set_discussion_resolved(DiscussionResolve {
      project_id: project,
      target,
      iid,
      discussion_id: discussion,
      actor_user_id: actor.user_id,
      can_write,
      resolved,
    })
    .await?;
  response(one(s, target, project, iid, discussion).await?)
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/issues/{iid}/discussions",tag="discussions",
    description="Issue comment threads with resolved state, oldest first.",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Discussion>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn issue_threads(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(json!(threads(&s, DiscussionTarget::Issue, p, i).await?))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/discussions",tag="discussions",
    description="Merge request comment threads with resolved state, oldest first.",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Discussion>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn merge_threads(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(json!(
    threads(&s, DiscussionTarget::MergeRequest, p, i).await?
  ))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/issues/{iid}/discussions/{discussion}/resolve",tag="discussions",
    params(api::IidPath,("discussion"=i64,Path,description="Discussion identifier")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Discussion>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn issue_resolve(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, d)): Path<(i64, i64, i64)>,
) -> Reply {
  resolve(&s, &h, p, DiscussionTarget::Issue, i, d, true).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/issues/{iid}/discussions/{discussion}/unresolve",tag="discussions",
    params(api::IidPath,("discussion"=i64,Path,description="Discussion identifier")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Discussion>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn issue_unresolve(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, d)): Path<(i64, i64, i64)>,
) -> Reply {
  resolve(&s, &h, p, DiscussionTarget::Issue, i, d, false).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/merge-requests/{iid}/discussions/{discussion}/resolve",tag="discussions",
    params(api::IidPath,("discussion"=i64,Path,description="Discussion identifier")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Discussion>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn merge_resolve(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, d)): Path<(i64, i64, i64)>,
) -> Reply {
  resolve(&s, &h, p, DiscussionTarget::MergeRequest, i, d, true).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/merge-requests/{iid}/discussions/{discussion}/unresolve",tag="discussions",
    params(api::IidPath,("discussion"=i64,Path,description="Discussion identifier")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Discussion>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn merge_unresolve(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, d)): Path<(i64, i64, i64)>,
) -> Reply {
  resolve(&s, &h, p, DiscussionTarget::MergeRequest, i, d, false).await
}

/// Fields collected from a comment request body.
struct Attach {
  discussion_id: Option<i64>,
  position: Option<String>,
}

fn attach(b: &Value) -> Result<Attach, AppError> {
  let discussion_id = match b.get("discussion_id") {
    None | Some(Value::Null) => None,
    Some(value) => Some(
      value
        .as_i64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::bad_request("discussion_id must be a positive integer"))?,
    ),
  };
  let position = match b.get("position") {
    None | Some(Value::Null) => None,
    Some(Value::String(text)) => Some(text.clone()),
    Some(value) if value.is_object() => Some(value.to_string()),
    Some(_) => return Err(AppError::bad_request("position must be a JSON object")),
  };
  Ok(Attach {
    discussion_id,
    position,
  })
}

/// Issue comment command, including the optional thread and inline anchor.
pub(super) fn issue_comment(
  b: &Value,
  author: i64,
  issue: i64,
  body: String,
) -> Result<IssueCommentCreate, AppError> {
  let Attach {
    discussion_id,
    position,
  } = attach(b)?;
  Ok(IssueCommentCreate {
    project_issue_id: issue,
    author_user_id: author,
    body,
    discussion_id,
    position,
  })
}

/// Merge request comment command with the same optional fields.
pub(super) fn merge_comment(
  b: &Value,
  author: i64,
  merge_request: i64,
  body: String,
) -> Result<MergeCommentCreate, AppError> {
  let Attach {
    discussion_id,
    position,
  } = attach(b)?;
  Ok(MergeCommentCreate {
    merge_request_id: merge_request,
    author_user_id: author,
    body,
    discussion_id,
    position,
  })
}
