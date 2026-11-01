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
  let mut assignees = collection(s, project, &iids, "project_issue_assignees").await?;
  let mut labels = collection(s, project, &iids, "project_issue_labels").await?;
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
  table: &str,
) -> Result<HashMap<i64, Vec<Value>>, AppError> {
  // The table is selected only by the two callers above. Join the issue table
  // so project-local IID collisions can never pull another project's rows.
  let mut sql = format!(
    "SELECT c.* FROM {table} c JOIN project_issues i ON i.id=c.project_issue_id WHERE i.project_id=?"
  );
  let mut args = vec![project.into()];
  if iids.len() == 1 {
    sql.push_str(" AND i.id=?");
    args.push((*iids.keys().next().expect("nonempty issues")).into());
  }
  sql.push_str(" ORDER BY c.created_at,c.id");
  let mut groups = HashMap::<i64, Vec<Value>>::new();
  for row in s.db.all(&sql, args).await? {
    let id = number(&row, "project_issue_id");
    if let Some(iid) = iids.get(&id) {
      groups
        .entry(id)
        .or_default()
        .push(super::issue_collaboration::child(row, *iid));
    }
  }
  Ok(groups)
}
