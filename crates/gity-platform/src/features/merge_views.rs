//! Merge request response shaping and request-field normalisation.
use super::*;
use gity_entities::project_milestones::{Column as MilestoneColumn, Entity as Milestones};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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
  let exists = Milestones::find()
    .filter(MilestoneColumn::ProjectId.eq(project))
    .filter(MilestoneColumn::Id.eq(id))
    .one(&s.db.connection)
    .await?;
  if exists.is_none() {
    return Err(AppError::not_found("milestone not found"));
  }
  Ok(json!(id))
}
