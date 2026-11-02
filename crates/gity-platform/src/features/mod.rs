//! Collaboration APIs. Every resource lookup is scoped to its project before mutation.
mod activity;
mod attachments;
mod checks;
mod codeowners;
mod discussions;
mod issue_collaboration;
mod issue_views;
mod issues;
mod issues_mutation;
mod labels;
mod merge_events;
mod paging;
pub(crate) use merge_events::recover as recover_merge_operations;
mod merge_requests;
mod merge_views;
mod milestones;
mod mr_collaboration;
mod project_lock;
mod releases;
mod rules;
mod subscriptions;
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
    .merge(milestones::documented_routes())
    .merge(labels::documented_routes())
    .merge(discussions::documented_routes())
    .merge(subscriptions::documented_routes())
    .merge(activity::documented_routes())
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
#[cfg(test)]
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
      state.collaboration.validate_users(&[id]).await?;
      out.push(id);
    }
  }
  Ok(out)
}
fn command<T: serde::de::DeserializeOwned>(mut value: Value) -> Result<T, AppError> {
  // Request timestamps use RFC 3339; persistence commands use UTC calendar values.
  if let Some(fields) = value.as_object_mut() {
    for (key, value) in fields.iter_mut() {
      if key.ends_with("_at")
        && let Some(raw) = value.as_str()
        && let Ok(stamp) = chrono::DateTime::parse_from_rfc3339(raw)
      {
        *value = json!(stamp.naive_utc());
      }
    }
  }
  serde_json::from_value(value).map_err(|error| AppError::bad_request(error.to_string()))
}
use gity_collaboration::domain::*;
use gity_persistence::model_value;
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
mod merge_patch_tests;

#[cfg(test)]
mod release_tests;

#[cfg(test)]
mod milestone_tests;

#[cfg(test)]
mod label_tests;

#[cfg(test)]
mod token_access_tests;

#[cfg(test)]
mod attachment_http_tests;

#[cfg(test)]
mod rule_tests;

#[cfg(test)]
mod collaboration_tests;

#[cfg(test)]
mod subscription_notification_tests;

#[cfg(test)]
mod subscription_tests;

#[cfg(test)]
mod discussion_tests;

#[cfg(test)]
mod activity_tests;
