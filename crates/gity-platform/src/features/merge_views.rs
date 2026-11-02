//! Merge request response shaping and request-field normalisation.
use super::*;

pub(super) fn view(mut row: Value) -> Value {
  for key in ["squash", "draft"] {
    let value = number(&row, key) != 0;
    row[key] = json!(value);
  }
  row
}

pub(super) fn flag(v: &Value, key: &str) -> i64 {
  v.get(key).and_then(Value::as_bool).map_or(0, |v| v as i64)
}

pub(super) async fn milestone_patch(
  s: &AppState,
  project: i64,
  v: &Value,
  key: &str,
) -> Result<Value, AppError> {
  if v.get(key).is_none_or(Value::is_null) {
    return Ok(Value::Null);
  }
  let id = number(v, key);
  if id <= 0 {
    return Err(AppError::bad_request(format!("{key} must be positive")));
  }
  s.collaboration.milestone_id(project, id).await?;
  Ok(json!(id))
}
