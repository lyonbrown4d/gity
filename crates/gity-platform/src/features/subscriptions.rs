//! Issue and merge request subscriptions plus per-user notification switches.
//!
//! Subscriptions are personal: the authenticated user always owns the row, and
//! reads still require project read access.
use super::subscription_views::{preference_view, state_view, subscription_view};
use super::*;
use crate::openapi::{schemas as api, subscription_schemas as spec};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};
use gity_collaboration::domain::{
  ISSUE_TARGET, MERGE_REQUEST_TARGET, NotificationPreferenceChange, SubscriptionCommand,
  SubscriptionFilter,
};
use std::collections::HashMap;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .nest(
      "/projects",
      utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
          issue_subscription,
          subscribe_issue,
          unsubscribe_issue
        ))
        .routes(utoipa_axum::routes!(
          merge_request_subscription,
          subscribe_merge_request,
          unsubscribe_merge_request
        )),
    )
    .routes(utoipa_axum::routes!(list))
    .routes(utoipa_axum::routes!(preferences, update_preference))
}

/// `subscribe` is `None` for the status read, `Some(true)` to subscribe and
/// `Some(false)` to unsubscribe; the target is always resolved through `iid`.
async fn project_subscription(
  s: &AppState,
  h: &HeaderMap,
  project: i64,
  iid: i64,
  target_type: &'static str,
  action: ProjectAction,
  subscribe: Option<bool>,
) -> Reply {
  if target_type == ISSUE_TARGET {
    super::issue_access::authorize(s, h, project, iid, action).await?;
  }
  let actor = crate::access::project_action(s, h, project, action)
    .await?
    .actor()?;
  let command = SubscriptionCommand {
    project,
    user: actor.user_id,
    target_type: target_type.to_owned(),
    iid,
  };
  let state = match subscribe {
    Some(true) => s.collaboration.subscribe(command).await?,
    Some(false) => s.collaboration.unsubscribe(command).await?,
    None => s.collaboration.subscription(command).await?,
  };
  response(state_view(state))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/issues/{iid}/subscription",tag="subscriptions",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::SubscriptionState>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn issue_subscription(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  project_subscription(&s, &h, p, i, ISSUE_TARGET, ProjectAction::Read, None).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    put,path="/{id}/issues/{iid}/subscription",tag="subscriptions",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::SubscriptionState>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn subscribe_issue(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  let action = ProjectAction::IssueComment;
  project_subscription(&s, &h, p, i, ISSUE_TARGET, action, Some(true)).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete,path="/{id}/issues/{iid}/subscription",tag="subscriptions",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::SubscriptionState>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn unsubscribe_issue(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  let action = ProjectAction::IssueComment;
  project_subscription(&s, &h, p, i, ISSUE_TARGET, action, Some(false)).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/subscription",tag="subscriptions",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::SubscriptionState>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn merge_request_subscription(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  let target = MERGE_REQUEST_TARGET;
  project_subscription(&s, &h, p, i, target, ProjectAction::Read, None).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    put,path="/{id}/merge-requests/{iid}/subscription",tag="subscriptions",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::SubscriptionState>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn subscribe_merge_request(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  let target = MERGE_REQUEST_TARGET;
  let action = ProjectAction::MergeRequestComment;
  project_subscription(&s, &h, p, i, target, action, Some(true)).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete,path="/{id}/merge-requests/{iid}/subscription",tag="subscriptions",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::SubscriptionState>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn unsubscribe_merge_request(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  let target = MERGE_REQUEST_TARGET;
  let action = ProjectAction::MergeRequestComment;
  project_subscription(&s, &h, p, i, target, action, Some(false)).await
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/subscriptions",tag="subscriptions",
    description="Lists the authenticated user's subscriptions, optionally filtered.",
    params(
      ("target_type"=Option<String>, Query, description="issue or merge_request"),
      ("target_id"=Option<String>, Query, description="Application-generated target identifier")
    ),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Subscription>>),api::Errors),
    security(("bearer"=[]))
)]
async fn list(
  State(s): State<AppState>,
  h: HeaderMap,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  let rows = s
    .collaboration
    .subscriptions(actor.user_id, filter(&q)?)
    .await?;
  let rows = super::resource_visibility::filter_subscriptions(&s, &h, rows).await?;
  response(json!(
    rows.into_iter().map(subscription_view).collect::<Vec<_>>()
  ))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/users/me/notification-preferences",tag="subscriptions",
    description="Lists the authenticated user's effective notification switches.",
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::NotificationPreference>>),api::Errors),
    security(("bearer"=[]))
)]
async fn preferences(State(s): State<AppState>, h: HeaderMap) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  let states = s
    .collaboration
    .notification_preferences(actor.user_id)
    .await?;
  response(json!(
    states.into_iter().map(preference_view).collect::<Vec<_>>()
  ))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    put,path="/users/me/notification-preferences",tag="subscriptions",
    description="Stores one notification switch for the authenticated user.",
    request_body(content=spec::UpdateNotificationPreference),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::NotificationPreference>),api::Errors),
    security(("bearer"=[]))
)]
async fn update_preference(State(s): State<AppState>, h: HeaderMap, Json(b): Json<Value>) -> Reply {
  let actor = crate::access::actor(&s, &h).await?;
  let enabled = b
    .get("enabled")
    .and_then(Value::as_bool)
    .ok_or_else(|| AppError::bad_request("enabled must be a boolean"))?;
  let change = NotificationPreferenceChange {
    kind: required(&b, "kind")?,
    enabled,
  };
  let state = s
    .collaboration
    .set_notification_preference(actor.user_id, change)
    .await?;
  response(preference_view(state))
}

fn filter(q: &HashMap<String, String>) -> Result<SubscriptionFilter, AppError> {
  let target_type = q
    .get("target_type")
    .map(|value| value.trim())
    .filter(|value| !value.is_empty())
    .map(str::to_owned);
  let target_id = match q
    .get("target_id")
    .map(|value| value.trim())
    .filter(|value| !value.is_empty())
  {
    Some(raw) => Some(
      raw
        .parse::<i64>()
        .map_err(|_| AppError::bad_request("target_id must be a positive integer"))?,
    ),
    None => None,
  };
  Ok(SubscriptionFilter {
    target_type,
    target_id,
  })
}
