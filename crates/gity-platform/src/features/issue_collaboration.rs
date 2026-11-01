use super::*;
use crate::openapi::{issue_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
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
pub(super) async fn children(
  s: &AppState,
  p: i64,
  i: i64,
  table: &str,
) -> Result<Vec<Value>, AppError> {
  let row = scoped(s, "project_issues", p, i, true).await?;
  Ok(
    s.db
      .all(
        &format!("SELECT * FROM {table} WHERE project_issue_id = ? ORDER BY created_at, id"),
        vec![number(&row, "id").into()],
      )
      .await?
      .into_iter()
      .map(|r| child(r, i))
      .collect(),
  )
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
  response(json!(children(&s, p, i, "project_issue_comments").await?))
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
  response(json!(children(&s, p, i, "project_issue_assignees").await?))
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
  response(json!(children(&s, p, i, "project_issue_labels").await?))
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
  let issue = scoped(&s, "project_issues", p, i, true).await?;
  let body = if string(&b, "body").is_empty() {
    required(&b, "content")?
  } else {
    required(&b, "body")?
  };
  let row = s
    .db
    .insert(
      "project_issue_comments",
      fields(
        json!({"project_issue_id":number(&issue,"id"),"author_user_id":actor.user_id,"body":body}),
      ),
    )
    .await?;
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
  let row = scoped(&s, "project_issues", p, i, true).await?;
  let ids = users(&s, &b["user_ids"]).await?;
  let id = number(&row, "id");
  let records = ids
    .iter()
    .map(|u| fields(json!({"project_issue_id":id,"user_id":u})))
    .collect();
  replace(
    &s,
    "project_issue_assignees",
    "project_issue_id",
    id,
    None,
    records,
  )
  .await?;
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
  response(json!(children(&s, p, i, "project_issue_assignees").await?))
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
  let row = scoped(&s, "project_issues", p, i, true).await?;
  let id = number(&row, "id");
  let mut labels = Vec::new();
  let mut seen = std::collections::HashSet::new();
  let input = b["labels"]
    .as_array()
    .cloned()
    .unwrap_or_default()
    .into_iter()
    .chain(["names", "label_names"].into_iter().flat_map(|k| {
      b[k]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| json!({"name":v}))
    }));
  for label in input {
    let name = required(&label, "name")?;
    if seen.insert(name.clone()) {
      labels.push(fields(
        json!({"project_issue_id":id,"name":name,"color":string(&label,"color").trim()}),
      ));
    }
  }
  replace(
    &s,
    "project_issue_labels",
    "project_issue_id",
    id,
    None,
    labels,
  )
  .await?;
  response(json!(children(&s, p, i, "project_issue_labels").await?))
}
