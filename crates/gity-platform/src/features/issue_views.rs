use super::*;
use std::collections::HashMap;

pub(super) async fn one(s: &AppState, project: i64, row: Value) -> Result<Value, AppError> {
  list(s, project, vec![row])
    .await?
    .pop()
    .ok_or_else(|| AppError::not_found("resource not found"))
}
pub(super) async fn list(
  s: &AppState,
  project: i64,
  rows: Vec<Value>,
) -> Result<Vec<Value>, AppError> {
  if rows.is_empty() {
    return Ok(Vec::new());
  }
  let iids = rows
    .iter()
    .map(|row| (number(row, "id"), number(row, "iid")))
    .collect::<HashMap<_, _>>();
  let mut assignees = collection(s, project, &iids).await?;
  let mut labels = label_rows(s, project, &iids).await?;
  Ok(
    rows
      .into_iter()
      .map(|row| {
        let id = number(&row, "id");
        let assigned = assignees.remove(&id).unwrap_or_default();
        let ids = assigned
          .iter()
          .map(|row| number(row, "user_id").to_string())
          .collect::<Vec<_>>();
        let mut view = super::issues::view(row);
        view["assignee_user_id"] = json!(ids.first());
        view["assignee_user_ids"] = json!(ids);
        view["assignees"] = json!(assigned);
        view["labels"] = json!(labels.remove(&id).unwrap_or_default());
        view
      })
      .collect(),
  )
}
async fn collection(
  s: &AppState,
  project: i64,
  iids: &HashMap<i64, i64>,
) -> Result<HashMap<i64, Vec<Value>>, AppError> {
  let mut groups = HashMap::<i64, Vec<Value>>::new();
  for model in s
    .collaboration
    .assigned_issues(project, iids.keys().copied().collect())
    .await?
  {
    let id = model.project_issue_id;
    if let Some(iid) = iids.get(&id) {
      groups
        .entry(id)
        .or_default()
        .push(super::issue_collaboration::child(model_value(model), *iid));
    }
  }
  Ok(groups)
}

/// Issue labels grouped by project-scoped issue primary key.
///
/// Links and definitions are read through the entities so a label attached to
/// an issue can never come from another project, and the ordering matches the
/// dedicated issue-label endpoint.
pub(super) async fn label_rows(
  s: &AppState,
  project: i64,
  iids: &HashMap<i64, i64>,
) -> Result<HashMap<i64, Vec<Value>>, AppError> {
  let mut groups = HashMap::<i64, Vec<Value>>::new();
  for (issue, model) in s
    .collaboration
    .issue_label_rows(project, iids.keys().copied().collect())
    .await?
  {
    if let Some(iid) = iids.get(&issue) {
      groups
        .entry(issue)
        .or_default()
        .push(super::labels::issue_view(&model, *iid));
    }
  }
  Ok(groups)
}

/// One entry of the merged issue timeline.
///
/// `type` distinguishes user comments from system events so the client can
/// render both from one sorted list.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub(super) struct TimelineEntry {
  /// Comment id or audit event id, serialized as a string.
  pub id: String,
  /// `comment` for user comments, `system` for audit events.
  #[serde(rename = "type")]
  #[schema(example = "comment")]
  pub entry_type: String,
  /// Comment author or the actor of the audited change.
  pub author_user_id: String,
  /// Comment text; null for system entries.
  pub body: Option<String>,
  /// Human readable system summary; null for comments.
  pub summary: Option<String>,
  /// Audit event name such as `project.issue.closed`; null for comments.
  pub event_name: Option<String>,
  #[schema(format = DateTime)]
  pub created_at: String,
}

/// Comments and audit events of one issue interleaved by creation time.
///
/// Snowflake identifiers are monotonic per process, so they break timestamp
/// ties in the order the rows were written.
pub(super) async fn timeline(s: &AppState, project: i64, iid: i64) -> Result<Vec<Value>, AppError> {
  let (comments, events) = s.collaboration.issue_timeline(project, iid).await?;
  let mut rows = comments
    .into_iter()
    .map(|comment| (comment.created_at, comment.id, comment_entry(&comment)))
    .chain(
      events
        .into_iter()
        .map(|event| (event.created_at, event.id, event_entry(&event))),
    )
    .collect::<Vec<_>>();
  rows.sort_by_key(|a| (a.0, a.1));
  Ok(rows.into_iter().map(|row| row.2).collect())
}

fn comment_entry(comment: &gity_entities::project_issue_comments::Model) -> Value {
  json!({
    "id": comment.id,
    "type": "comment",
    "author_user_id": comment.author_user_id,
    "body": comment.body,
    "summary": Value::Null,
    "event_name": Value::Null,
    "created_at": comment.created_at,
  })
}

fn event_entry(event: &gity_entities::project_audit_events::Model) -> Value {
  json!({
    "id": event.id,
    "type": "system",
    "author_user_id": event.actor_user_id,
    "body": Value::Null,
    "summary": event.summary,
    "event_name": event.event_name,
    "created_at": event.created_at,
  })
}
