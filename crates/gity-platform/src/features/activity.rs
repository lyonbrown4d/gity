//! Cross-project activity feed for the signed-in user.
//!
//! The feed is personal: the actor is always the authenticated user, and the
//! repository decides which projects count as readable.
use super::*;
use crate::openapi::{activity_schemas as spec, schemas as api};
use axum::{
  extract::{Query, State},
  http::HeaderMap,
};
use gity_collaboration::activity::ActivityItem;
use std::collections::HashMap;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(list))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/activity",tag="activity",
    params(("limit"=Option<i64>,Query,description="1–100; defaults to 50")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ActivityFeed>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(
  State(s): State<AppState>,
  h: HeaderMap,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  let limit = q
    .get("limit")
    .and_then(|value| value.parse::<u64>().ok())
    .filter(|value| *value > 0)
    .map(|value| value.min(100))
    .unwrap_or(50);
  let items = s
    .collaboration
    .activity(actor.user_id, limit)
    .await?
    .into_iter()
    .map(view)
    .collect::<Vec<_>>();
  response(json!({"items":items}))
}
fn view(item: ActivityItem) -> Value {
  let event = item.event;
  json!({
    "id":event.id.to_string(),
    "event_name":event.event_name,
    "action":event.action,
    "actor_user_id":event.actor_user_id.to_string(),
    "actor_name":item.actor_name,
    "target_type":event.target_type,
    "target_id":event.target_id,
    "summary":event.summary,
    "project_id":event.project_id.to_string(),
    "project_name":item.project_name,
    "organization_id":event.organization_id.to_string(),
    "project_path":item.project_path,
    "created_at":event.created_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
  })
}
