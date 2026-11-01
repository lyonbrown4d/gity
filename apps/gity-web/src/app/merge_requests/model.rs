use crate::api;
use serde_json::Value;

pub(super) fn filter(
  rows: &[Value],
  state: &str,
  search: &str,
  source: &str,
  target: &str,
) -> Vec<Value> {
  let search = search.trim().to_lowercase();
  rows
    .iter()
    .filter(|row| {
      (state == "all" || api::text(row, "state") == state)
        && (source.is_empty() || api::text(row, "source_branch") == source)
        && (target.is_empty() || api::text(row, "target_branch") == target)
        && (search.is_empty()
          || [
            "iid",
            "title",
            "description",
            "source_branch",
            "target_branch",
          ]
          .iter()
          .any(|key| api::text(row, key).to_lowercase().contains(&search)))
    })
    .cloned()
    .collect()
}
pub(super) fn count(value: &Value, state: &str) -> usize {
  api::list(value)
    .iter()
    .filter(|row| api::text(row, "state") == state)
    .count()
}
pub(super) fn list(value: &Value, key: &str) -> Vec<Value> {
  value[key].as_array().cloned().unwrap_or_default()
}
pub(super) fn user(users: &Value, id: &str) -> String {
  api::list(users)
    .iter()
    .find(|row| api::text(row, "id") == id)
    .map(|row| {
      let name = api::text(row, "display_name");
      let username = api::text(row, "username");
      if name.trim().is_empty() {
        format!("@{username}")
      } else {
        format!("{} (@{username})", name.trim())
      }
    })
    .unwrap_or_else(|| format!("#{id}"))
}
pub(super) fn state_label(state: &str) -> &'static str {
  match state {
    "opened" => "Open",
    "merged" => "Merged",
    _ => "Closed",
  }
}
pub(super) fn ids(values: &[String]) -> Vec<i64> {
  values.iter().filter_map(|id| id.parse().ok()).collect()
}
pub(super) fn strings(value: &Value) -> Vec<String> {
  value
    .as_array()
    .into_iter()
    .flatten()
    .map(|v| {
      v.as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| v.to_string())
    })
    .collect()
}
pub(super) fn readiness(mr: &Value, checks: &Value) -> &'static str {
  match api::text(mr, "state").as_str() {
    "merged" => "Merged",
    "closed" => "Closed",
    _ if checks["mergeable"] == true => "Ready to merge",
    _ if checks["required"] == true => "Blocked",
    _ => "Review in progress",
  }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
