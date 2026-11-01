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
    let user =
        s.db.one(
            "SELECT username,display_name FROM users WHERE id = ?",
            vec![num(row, "owner_user_id").into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("lock owner not found"))?;
    Ok(
        json!({"id":num(row,"id").to_string(),"path":row["path"],"locked_at":row["created_at"],"owner":{"name":if text(&user,"display_name").is_empty(){text(&user,"username")}else{text(&user,"display_name")}}}),
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
    let sql = concat!(
        "SELECT * FROM project_lfs_locks WHERE project_id = ? AND id > ? ",
        "AND (? = '' OR path = ?) AND (? = 0 OR id = ?) ORDER BY id LIMIT ?"
    );
    let args = vec![
        p.into(),
        cursor.into(),
        filter.clone().into(),
        filter.into(),
        id.into(),
        id.into(),
        (limit + 1).into(),
    ];
    let mut rows = s.db.all(sql, args).await?;
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
        s.db.insert(
            "project_lfs_locks",
            map(json!({"project_id":p,"owner_user_id":user,"path":path})),
        )
        .await?;
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
    let row = scoped(s, "project_lfs_locks", p, id).await?;
    if num(&row, "owner_user_id") != user && b["force"] != true {
        return Err(AppError::forbidden("lfs lock is owned by another user"));
    }
    let value = view(s, &row).await?;
    s.db.delete("project_lfs_locks", id).await?;
    Ok(json!({"lock":value}))
}
#[cfg(test)]
#[path = "lfs_locks_tests.rs"]
mod tests;
