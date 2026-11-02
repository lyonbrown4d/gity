use super::*;
use crate::openapi::{issue_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
use std::collections::{HashMap, HashSet};
pub(super) fn child(mut row: Value, iid: i64) -> Value {
  if let Some(map) = row.as_object_mut() {
    map.remove("project_issue_id");
    map.insert("issue_id".into(), json!(iid.to_string()));
    if let Some(body) = map.remove("body") {
      map.insert("content".into(), body);
    }
  }
  row
}
#[derive(Clone, Copy)]
pub(super) enum IssueCollection {
  Comments,
  Assignees,
}
pub(super) async fn children(
  s: &AppState,
  p: i64,
  i: i64,
  collection: IssueCollection,
) -> Result<Vec<Value>, AppError> {
  let rows = match collection {
    IssueCollection::Comments => s
      .collaboration
      .issue_comments(p, i)
      .await?
      .into_iter()
      .map(model_value)
      .collect::<Vec<_>>(),
    IssueCollection::Assignees => s
      .collaboration
      .issue_assignees(p, i)
      .await?
      .into_iter()
      .map(model_value)
      .collect::<Vec<_>>(),
  };
  Ok(rows.into_iter().map(|r| child(r, i)).collect())
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}/comments",tag="issues",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Comment>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn comments(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(json!(children(&s, p, i, IssueCollection::Comments).await?))
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}/assignees",tag="issues",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Assignee>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn assignees(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(json!(children(&s, p, i, IssueCollection::Assignees).await?))
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}/labels",tag="issues",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Label>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn labels(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(json!(issue_labels(&s, p, i).await?))
}
/// Labels of one issue, resolved through project definitions and label links.
async fn issue_labels(s: &AppState, p: i64, i: i64) -> Result<Vec<Value>, AppError> {
  let row = model_value(s.collaboration.issue(p, i).await?);
  let id = number(&row, "id");
  let mut groups = super::issue_views::label_rows(s, p, &HashMap::from([(id, i)])).await?;
  Ok(groups.remove(&id).unwrap_or_default())
}
#[utoipa::path(
    post,path="/{id}/issues/{iid}/comments",tag="issues",
    params(api::IidPath),
    request_body(content=spec::CreateComment),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Comment>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn comment(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::IssueComment)
    .await?
    .actor()?;
  let issue = model_value(&s.collaboration.issue(p, i).await?);
  let body = if string(&b, "body").is_empty() {
    required(&b, "content")?
  } else {
    required(&b, "body")?
  };
  let row = model_value(
    s.collaboration
      .create_issue_comment(
        p,
        i,
        IssueCommentCreate {
          project_issue_id: number(&issue, "id"),
          author_user_id: actor.user_id,
          body,
        },
      )
      .await?,
  );
  if number(&issue, "author_user_id") != actor.user_id {
    super::todos::notify(
      &s,
      super::todos::Notification {
        project: p,
        user: number(&issue, "author_user_id"),
        kind: "issue_commented",
        target_type: "issue_comment",
        target: number(&row, "id"),
        title: &format!("New comment on issue #{i}"),
        summary: string(&issue, "title"),
        suffix: &format!("/issues/{i}"),
      },
    )
    .await?;
  }
  response(child(row, i))
}
#[utoipa::path(
    patch,path="/{id}/issues/{iid}/assignees",tag="issues",
    params(api::IidPath),
    request_body(content=api::UserIds),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Assignee>>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn set_assignees(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite)
    .await?
    .actor()?;
  let row = model_value(&s.collaboration.issue(p, i).await?);
  let ids = users(&s, &b["user_ids"]).await?;
  let id = number(&row, "id");
  s.collaboration.replace_assignees(p, i, ids.clone()).await?;
  for u in ids {
    if u != actor.user_id {
      super::todos::notify(
        &s,
        super::todos::Notification {
          project: p,
          user: u,
          kind: "issue_assigned",
          target_type: "issue",
          target: id,
          title: &format!("Assigned to issue #{i}"),
          summary: string(&row, "title"),
          suffix: &format!("/issues/{i}"),
        },
      )
      .await?;
    }
  }
  response(json!(children(&s, p, i, IssueCollection::Assignees).await?))
}
#[utoipa::path(
    patch,path="/{id}/issues/{iid}/labels",tag="issues",
    params(api::IidPath),
    request_body(content=spec::Labels),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Label>>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn set_labels(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  s.collaboration.replace_labels(p, i, requested(&b)?).await?;
  response(json!(issue_labels(&s, p, i).await?))
}
/// Accepts the three historical shapes: label objects plus the bare
/// `names`/`label_names` lists used while labels were inlined per issue.
fn requested(body: &Value) -> Result<Vec<(String, Option<String>)>, AppError> {
  let objects = body["labels"]
    .as_array()
    .cloned()
    .unwrap_or_default()
    .into_iter()
    .map(|row| (row["name"].clone(), string(&row, "color").trim().to_owned()));
  let names = ["names", "label_names"]
    .into_iter()
    .flat_map(|key| body[key].as_array().cloned().unwrap_or_default())
    .map(|name| (name, String::new()));
  let mut out = Vec::new();
  let mut seen = HashSet::new();
  for (value, color) in objects.chain(names) {
    let title = value.as_str().unwrap_or("").trim().to_owned();
    if title.is_empty() {
      return Err(AppError::bad_request("name is required"));
    }
    if seen.insert(title.clone()) {
      out.push((title, (!color.is_empty()).then_some(color)));
    }
  }
  Ok(out)
}
