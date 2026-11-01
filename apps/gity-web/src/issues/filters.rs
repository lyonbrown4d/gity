use crate::api;
use serde_json::Value;

pub(super) fn labels(issue: &Value) -> Vec<String> {
  issue["labels"]
    .as_array()
    .into_iter()
    .flatten()
    .map(|v| api::text(v, "name"))
    .collect()
}
pub(super) fn assignees(issue: &Value) -> Vec<String> {
  let all = issue["assignee_user_ids"]
    .as_array()
    .into_iter()
    .flatten()
    .filter_map(Value::as_str)
    .map(str::to_string)
    .collect::<Vec<_>>();
  if all.is_empty() {
    let one = api::text(issue, "assignee_user_id");
    if one.is_empty() { all } else { vec![one] }
  } else {
    all
  }
}
