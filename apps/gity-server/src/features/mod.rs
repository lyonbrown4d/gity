//! Collaboration APIs. Every resource lookup is scoped to its project before mutation.
mod attachments;
mod checks;
mod codeowners;
mod issue_collaboration;
mod issue_views;
mod issues;
mod merge_events;
mod merge_requests;
mod mr_collaboration;
mod project_lock;
mod releases;
mod rules;
mod tags;
mod todos;
pub(crate) use todos::{Notification, notify as notify_event};
mod wiki;

use crate::{AppState, access::ProjectAction};
use axum::{Json, Router};
use gity_core::{AppError, externalize, now};
use serde_json::{Map, Value, json};

type Reply = Result<Json<Value>, AppError>;
pub fn routes() -> Router<AppState> {
    documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    issues::documented_routes()
        .merge(attachments::documented_routes())
        .merge(wiki::documented_routes())
        .merge(todos::documented_routes())
        .merge(merge_requests::documented_routes())
        .merge(rules::documented_routes())
        .merge(releases::documented_routes())
}
fn response(v: Value) -> Reply {
    Ok(gity_core::ok(externalize(v)))
}
fn number(v: &Value, key: &str) -> i64 {
    v[key]
        .as_i64()
        .or_else(|| v[key].as_str().and_then(|v| v.parse().ok()))
        .unwrap_or(0)
}
fn string<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn required(v: &Value, key: &str) -> Result<String, AppError> {
    let value = string(v, key).trim();
    if value.is_empty() {
        return Err(AppError::bad_request(format!("{key} is required")));
    }
    Ok(value.to_owned())
}
fn fields(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}
fn patch(body: &Value, keys: &[&str]) -> Map<String, Value> {
    let mut out = Map::new();
    for key in keys {
        if let Some(v) = body.get(*key)
            && !v.is_null()
        {
            out.insert((*key).to_owned(), v.clone());
        }
    }
    out.insert("updated_at".into(), json!(now()));
    out
}
async fn scoped(
    state: &AppState,
    table: &str,
    project: i64,
    id: i64,
    iid: bool,
) -> Result<Value, AppError> {
    state
        .db
        .one(
            &format!(
                "SELECT * FROM {table} WHERE project_id = ? AND {} = ?",
                if iid { "iid" } else { "id" }
            ),
            vec![project.into(), id.into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("resource not found"))
}
async fn users(state: &AppState, body: &Value) -> Result<Vec<i64>, AppError> {
    let mut out = Vec::new();
    if body.is_null() {
        return Ok(out);
    }
    for value in body
        .as_array()
        .ok_or_else(|| AppError::bad_request("user_ids must be an array"))?
    {
        let id = value
            .as_i64()
            .or_else(|| value.as_str().and_then(|v| v.parse().ok()))
            .unwrap_or(0);
        if id <= 0 {
            return Err(AppError::bad_request("user id must be positive"));
        }
        if !out.contains(&id) {
            if state
                .db
                .one("SELECT id FROM users WHERE id = ?", vec![id.into()])
                .await?
                .is_none()
            {
                return Err(AppError::not_found("user not found"));
            }
            out.push(id);
        }
    }
    Ok(out)
}
async fn replace(
    state: &AppState,
    table: &str,
    fk: &str,
    id: i64,
    role: Option<&str>,
    rows: Vec<Map<String, Value>>,
) -> Result<(), AppError> {
    use sea_orm::{ConnectionTrait, TransactionTrait};
    let tx = state.db.connection.begin().await.map_err(AppError::from)?;
    let mut args = vec![id.into()];
    let mut sql = format!("DELETE FROM {table} WHERE {fk} = ?");
    if let Some(role) = role {
        sql.push_str(" AND role = ?");
        args.push(role.into());
    }
    tx.execute_raw(state.db.statement(&sql, args))
        .await
        .map_err(AppError::from)?;
    for mut row in rows {
        row.insert("id".into(), json!(gity_core::new_id()));
        row.insert("created_at".into(), json!(now()));
        row.insert("updated_at".into(), json!(now()));
        let keys = row.keys().cloned().collect::<Vec<_>>();
        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            keys.join(","),
            vec!["?"; keys.len()].join(",")
        );
        let values = keys
            .iter()
            .map(|key| match &row[key] {
                Value::Number(n) => sea_orm::Value::BigInt(n.as_i64()),
                Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
                Value::String(s) => s.clone().into(),
                _ => sea_orm::Value::String(None),
            })
            .collect();
        tx.execute_raw(state.db.statement(&sql, values))
            .await
            .map_err(AppError::from)?;
    }
    tx.commit().await.map_err(AppError::from)
}
#[cfg(test)]
mod helper_tests;

#[cfg(test)]
mod http_tests;
#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod codeowners_http_tests;
#[cfg(test)]
mod merge_tests;

#[cfg(test)]
mod release_tests;

#[cfg(test)]
mod token_access_tests;

#[cfg(test)]
mod attachment_http_tests;
