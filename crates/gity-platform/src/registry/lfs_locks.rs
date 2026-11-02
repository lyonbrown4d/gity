use super::*;
use std::collections::HashMap;
pub(super) fn pagination(q: &HashMap<String, String>) -> Result<(i64, i64), AppError> {
  let cursor = q
    .get("cursor")
    .filter(|v| !v.is_empty())
    .map(|v| {
      v.parse::<i64>()
        .map_err(|_| AppError::bad_request("invalid cursor"))
    })
    .transpose()?
    .unwrap_or(0);
  if cursor < 0 {
    return Err(AppError::bad_request("invalid cursor"));
  }
  let limit = q
    .get("limit")
    .map(|v| {
      v.parse::<i64>()
        .map_err(|_| AppError::bad_request("invalid limit"))
    })
    .transpose()?
    .filter(|v| *v > 0 && *v <= 100)
    .unwrap_or(100);
  Ok((cursor, limit))
}
fn path(v: &str) -> Result<String, AppError> {
  let value = v.trim().replace('\\', "/");
  let value = value.trim_matches('/');
  if value.is_empty() || value.split('/').any(|p| ["", ".", ".."].contains(&p)) {
    return Err(AppError::bad_request("invalid lock path"));
  }
  Ok(value.to_owned())
}
async fn view(s: &AppState, row: &Value) -> Result<Value, AppError> {
  let user = s
    .repository_access
    .lfs_owner(num(row, "owner_user_id"))
    .await?;
  Ok(
    json!({"id":num(row,"id").to_string(),"path":row["path"],"locked_at":row["created_at"],"owner":{"name":if user.display_name.is_empty(){&user.username}else{&user.display_name}}}),
  )
}
pub(super) async fn list(
  s: &AppState,
  p: i64,
  q: &HashMap<String, String>,
  verify: Option<i64>,
) -> Result<Value, AppError> {
  let (cursor, limit) = pagination(q)?;
  let filter = q
    .get("path")
    .filter(|v| !v.is_empty())
    .map(|v| path(v))
    .transpose()?
    .unwrap_or_default();
  let id = q
    .get("id")
    .filter(|v| !v.is_empty())
    .map(|v| {
      v.parse::<i64>()
        .map_err(|_| AppError::bad_request("invalid lock id"))
    })
    .transpose()?
    .unwrap_or(0);
  let query = gity_projects::LockQuery {
    cursor,
    limit: (limit + 1) as u64,
    path: (!filter.is_empty()).then_some(filter),
    id: (id != 0).then_some(id),
  };
  let mut rows = s
    .repository_access
    .lfs_locks(p, query)
    .await?
    .into_iter()
    .map(gity_persistence::model_value)
    .collect::<Vec<_>>();
  let next = if rows.len() > limit as usize {
    rows.truncate(limit as usize);
    num(rows.last().unwrap(), "id").to_string()
  } else {
    String::new()
  };
  let mut ours = Vec::new();
  let mut theirs = Vec::new();
  for row in rows {
    let value = view(s, &row).await?;
    if verify.is_none() || verify == Some(num(&row, "owner_user_id")) {
      ours.push(value);
    } else {
      theirs.push(value);
    }
  }
  Ok(if verify.is_some() {
    json!({"ours":ours,"theirs":theirs,"next_cursor":next})
  } else {
    json!({"locks":ours,"next_cursor":next})
  })
}
pub(super) async fn create(s: &AppState, p: i64, user: i64, b: &Value) -> Result<Value, AppError> {
  let path = path(&required(b, "path")?)?;
  let row =
    gity_persistence::model_value(s.repository_access.create_lfs_lock(p, user, path).await?);
  Ok(json!({"lock":view(s,&row).await?}))
}
pub(super) async fn remove(
  s: &AppState,
  p: i64,
  user: i64,
  id: i64,
  b: &Value,
) -> Result<Value, AppError> {
  if id <= 0 {
    return Err(AppError::bad_request("invalid lock id"));
  }
  let row = gity_persistence::model_value(
    s.repository_access
      .delete_lfs_lock(p, user, id, b["force"] == true)
      .await?,
  );
  Ok(json!({"lock":view(s,&row).await?}))
}
#[cfg(test)]
#[path = "lfs_locks_tests.rs"]
mod tests;
