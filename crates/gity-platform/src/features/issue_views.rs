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
