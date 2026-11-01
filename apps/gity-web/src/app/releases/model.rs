use crate::api;
use serde_json::Value;

pub(super) fn active(rows: &Value, selected: &str) -> Value {
  api::list(rows)
    .into_iter()
    .find(|row| api::text(&row["release"], "id") == selected)
    .unwrap_or(Value::Null)
}
pub(super) fn links(value: &Value) -> Vec<Value> {
  value["links"].as_array().cloned().unwrap_or_default()
}
pub(super) fn link_count(rows: &Value) -> usize {
  api::list(rows).iter().map(|row| links(row).len()).sum()
}
pub(super) fn short_sha(value: &str) -> String {
  if value.is_empty() {
    "--------".into()
  } else {
    value.chars().take(8).collect()
  }
}
/// Only expose the same HTTP(S) protocols accepted by the release asset API.
pub(super) fn asset_url(value: &str) -> Option<String> {
  if value.chars().any(char::is_control) {
    return None;
  }
  let (scheme, rest) = value.split_once("://")?;
  if !["http", "https"]
    .iter()
    .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
  {
    return None;
  }
  let authority = rest.split(['/', '?', '#']).next()?;
  if authority.is_empty() || authority.chars().any(char::is_whitespace) {
    return None;
  }
  Some(value.into())
}
pub(super) fn release_body(
  tag: &str,
  name: &str,
  description: &str,
  source: &str,
  default_branch: &str,
  create_tag: bool,
) -> Value {
  let tag = tag.trim();
  let source = source.trim();
  serde_json::json!({
      "tag_name":tag,
      "name":if name.trim().is_empty() {tag} else {name.trim()},
      "description":description.trim(),
      "source_ref":if source.is_empty() {if default_branch.is_empty() {"main"} else {default_branch}} else {source},
      "create_tag":create_tag
  })
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
